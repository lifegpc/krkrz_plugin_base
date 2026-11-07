//! Asynchronous execution of closures on the Kirikiri Z (krkrz) window thread.
//!
//! [`AsyncExecutor`] must be constructed on the krkrz window thread (for example
//! inside `V2Link` or a TJS native function/constructor). Once constructed, it
//! can be cloned and sent across threads (`Clone + Send + Sync`), allowing
//! background threads or async runtimes to schedule closures onto the krkrz
//! window thread via [`AsyncExecutor::execute`].

use crate::tp_stub::*;
use krkrz_plugin_base_macros::tjs_w;
use std::any::Any;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::fmt;
use std::future::Future;
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::pin::Pin;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_POPUP: u32 = 0x8000_0000;
const GWLP_WNDPROC: i32 = -4;
const GWLP_USERDATA: i32 = -21;
const WM_APP: u32 = 0x8000;
const WM_SCHEDULE_EVENT: u32 = WM_APP + 1;
const WM_CLEANUP_EVENT: u32 = WM_APP + 2;

type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: Option<WndProc>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut c_void,
    h_icon: *mut c_void,
    h_cursor: *mut c_void,
    hbr_background: *mut c_void,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: *mut c_void,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
    fn GetLastError() -> u32;
    fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        wnd_parent: HWND,
        menu: *mut c_void,
        instance: *mut c_void,
        param: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    fn DestroyWindow(hwnd: HWND) -> i32;
    fn GetClassInfoExW(
        instance: *mut c_void,
        class_name: *const u16,
        wnd_class: *mut WNDCLASSEXW,
    ) -> i32;
    fn PostMessageW(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> i32;
    fn RegisterClassExW(wnd_class: *const WNDCLASSEXW) -> u16;
}

#[cfg(target_pointer_width = "64")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: HWND, index: i32, new_long: isize) -> isize;
}

#[cfg(target_pointer_width = "32")]
#[link(name = "user32")]
unsafe extern "system" {
    #[link_name = "GetWindowLongW"]
    fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> isize;
    #[link_name = "SetWindowLongW"]
    fn SetWindowLongPtrW(hwnd: HWND, index: i32, new_long: isize) -> isize;
}

type BoxedTask = Box<dyn FnOnce() + Send + 'static>;

struct SharedQueue {
    queue: Mutex<VecDeque<BoxedTask>>,
    hwnd: AtomicPtr<HWND__>,
    event_posted: AtomicBool,
}

impl SharedQueue {
    fn notify(&self) {
        if !self.event_posted.swap(true, Ordering::AcqRel) {
            let hwnd = self.hwnd.load(Ordering::Acquire);
            if !hwnd.is_null() {
                unsafe {
                    PostMessageW(hwnd, WM_SCHEDULE_EVENT, 0, 0);
                }
            }
        }
    }

    fn drain_tasks(&self) {
        self.event_posted.store(false, Ordering::Release);
        loop {
            let task = self.queue.lock().unwrap().pop_front();
            match task {
                Some(task) => task(),
                None => break,
            }
        }
        if !self.queue.lock().unwrap().is_empty() {
            self.notify();
        }
    }
}

struct EventTarget {
    shared: Arc<SharedQueue>,
}

impl TJSDispatch for EventTarget {
    fn func_call(
        &mut self,
        _flag: tjs_uint32,
        _membername: *const tjs_char,
        _hint: *mut tjs_uint32,
        result: *mut tTJSVariant,
        _numparams: tjs_int,
        _param: *mut *mut tTJSVariant,
        _objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        self.shared.drain_tasks();
        if let Some(result) = unsafe { result.as_mut() } {
            result.clear();
        }
        TJS_S_OK
    }

    fn is_valid(
        &mut self,
        _flag: tjs_uint32,
        membername: *const tjs_char,
        _hint: *mut tjs_uint32,
        _objthis: *mut iTJSDispatch2,
    ) -> tjs_error {
        if membername.is_null() {
            TJS_S_TRUE
        } else {
            TJS_E_MEMBERNOTFOUND
        }
    }
}

/// Window-thread-only context stored in `GWLP_USERDATA` of the hidden message window.
struct WndContext {
    dispatch: *mut iTJSDispatch2,
    event_name: ttstr,
    shared: Arc<SharedQueue>,
}

impl Drop for WndContext {
    fn drop(&mut self) {
        if !self.dispatch.is_null() {
            unsafe {
                TVPCancelSourceEvents(self.dispatch);
                (*self.dispatch).release();
            }
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_SCHEDULE_EVENT {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WndContext;
        if let Some(ctx) = unsafe { ptr.as_mut() } {
            unsafe {
                TVPPostEvent(
                    ctx.dispatch,
                    ctx.dispatch,
                    &mut ctx.event_name,
                    0,
                    (TVP_EPT_POST | TVP_EPT_REMOVE_POST) as u32,
                    0,
                    ptr::null_mut(),
                );
            }
        }
        return 0;
    } else if msg == WM_CLEANUP_EVENT {
        unsafe { destroy_window_and_context(hwnd) };
        return 0;
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe fn destroy_window_and_context(hwnd: HWND) {
    if hwnd.is_null() {
        return;
    }
    let ptr = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) } as *mut WndContext;
    unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWLP_WNDPROC,
            DefWindowProcW as *const () as usize as isize,
        );
        DestroyWindow(hwnd);
    }
    if !ptr.is_null() {
        let ctx = unsafe { Box::from_raw(ptr) };
        ctx.shared.drain_tasks();
        drop(ctx);
    }
}

fn create_queue_window(ctx: Box<WndContext>) -> io::Result<HWND> {
    let class_name = tjs_w!("TVPQueueWindow");
    let empty_name = tjs_w!("");
    unsafe {
        let h_instance = GetModuleHandleW(ptr::null());
        let mut tmp_wc: WNDCLASSEXW = std::mem::zeroed();
        tmp_wc.cb_size = size_of::<WNDCLASSEXW>() as u32;

        if GetClassInfoExW(h_instance, class_name, &mut tmp_wc) == 0 {
            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cb_size = size_of::<WNDCLASSEXW>() as u32;
            wc.lpfn_wnd_proc = Some(DefWindowProcW);
            wc.h_instance = h_instance;
            wc.lpsz_class_name = class_name;
            if RegisterClassExW(&wc) == 0 {
                let err = GetLastError();
                if GetClassInfoExW(h_instance, class_name, &mut tmp_wc) == 0 {
                    return Err(io::Error::from_raw_os_error(err as i32));
                }
            }
        }

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name,
            empty_name,
            WS_POPUP,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            h_instance,
            ptr::null_mut(),
        );
        if hwnd.is_null() {
            return Err(io::Error::from_raw_os_error(GetLastError() as i32));
        }

        let raw_ctx = Box::into_raw(ctx);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw_ctx as isize);
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wnd_proc as *const () as usize as isize);
        Ok(hwnd)
    }
}

struct ExecutorInner {
    shared: Arc<SharedQueue>,
    window_thread_id: u32,
}

impl Drop for ExecutorInner {
    fn drop(&mut self) {
        let hwnd = self.shared.hwnd.swap(ptr::null_mut(), Ordering::AcqRel);
        if hwnd.is_null() {
            return;
        }
        let current_thread_id = unsafe { GetCurrentThreadId() };
        if current_thread_id == self.window_thread_id {
            unsafe { destroy_window_and_context(hwnd) };
        } else {
            unsafe {
                PostMessageW(hwnd, WM_CLEANUP_EVENT, 0, 0);
            }
        }
    }
}

/// An executor that dispatches closures onto the Kirikiri Z (krkrz) window thread.
///
/// Create an [`AsyncExecutor`] on the krkrz window thread via [`AsyncExecutor::new`]
/// or [`AsyncExecutor::try_new`], then clone it and pass it to async tasks or worker
/// threads to run closures on the window thread with [`AsyncExecutor::execute`].
#[derive(Clone)]
pub struct AsyncExecutor {
    inner: Arc<ExecutorInner>,
}

impl fmt::Debug for AsyncExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncExecutor")
            .field("window_thread_id", &self.inner.window_thread_id)
            .finish()
    }
}

impl Default for AsyncExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncExecutor {
    /// Creates a new [`AsyncExecutor`] bound to the current thread (which must be
    /// the krkrz window thread).
    ///
    /// # Panics
    ///
    /// Panics if creating the internal Win32 message queue window fails.
    pub fn new() -> Self {
        Self::try_new().expect("failed to create krkrz window thread executor")
    }

    /// Attempts to create a new [`AsyncExecutor`] bound to the current thread
    /// (which must be the krkrz window thread).
    pub fn try_new() -> io::Result<Self> {
        let window_thread_id = unsafe { GetCurrentThreadId() };
        let shared = Arc::new(SharedQueue {
            queue: Mutex::new(VecDeque::new()),
            hwnd: AtomicPtr::new(ptr::null_mut()),
            event_posted: AtomicBool::new(false),
        });

        let dispatch = tTJSDispatch::new(EventTarget {
            shared: Arc::clone(&shared),
        });
        let event_name = ttstr::from(tjs_w!("onExecute"));
        let ctx = Box::new(WndContext {
            dispatch,
            event_name,
            shared: Arc::clone(&shared),
        });

        match create_queue_window(ctx) {
            Ok(hwnd) => {
                shared.hwnd.store(hwnd, Ordering::Release);
                Ok(Self {
                    inner: Arc::new(ExecutorInner {
                        shared,
                        window_thread_id,
                    }),
                })
            }
            Err(err) => Err(err),
        }
    }

    /// Returns the OS thread ID of the krkrz window thread this executor is bound to.
    #[inline]
    pub fn window_thread_id(&self) -> u32 {
        self.inner.window_thread_id
    }

    /// Returns `true` if the current thread is the krkrz window thread this executor
    /// is bound to.
    #[inline]
    pub fn is_window_thread(&self) -> bool {
        unsafe { GetCurrentThreadId() == self.inner.window_thread_id }
    }

    /// Executes `script` on the krkrz window thread and awaits its return value.
    ///
    /// The closure is scheduled via the window thread's event queue (`TVPPostEvent`)
    /// and guaranteed to run on the krkrz window thread. If `script` panics, the
    /// panic is caught before crossing the FFI boundary and resumed on the task
    /// awaiting this method.
    pub async fn execute<F, R>(&self, script: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        TaskFuture {
            shared: Arc::clone(&self.inner.shared),
            script: Some(script),
            state: Arc::new(Mutex::new(TaskState {
                result: None,
                waker: None,
                cancelled: false,
            })),
        }
        .await
    }
}

struct TaskState<R> {
    result: Option<Result<R, Box<dyn Any + Send + 'static>>>,
    waker: Option<Waker>,
    cancelled: bool,
}

struct TaskFuture<F, R> {
    shared: Arc<SharedQueue>,
    script: Option<F>,
    state: Arc<Mutex<TaskState<R>>>,
}

impl<F, R> Unpin for TaskFuture<F, R> {}

impl<F, R> Future for TaskFuture<F, R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        if let Some(script) = this.script.take() {
            {
                let mut guard = this.state.lock().unwrap();
                guard.waker = Some(cx.waker().clone());
            }

            let task_state = Arc::clone(&this.state);
            let task: BoxedTask = Box::new(move || {
                if task_state.lock().unwrap().cancelled {
                    return;
                }
                let output = panic::catch_unwind(AssertUnwindSafe(script));
                let waker = {
                    let mut guard = task_state.lock().unwrap();
                    guard.result = Some(output);
                    guard.waker.take()
                };
                if let Some(waker) = waker {
                    waker.wake();
                }
            });

            this.shared.queue.lock().unwrap().push_back(task);
            this.shared.notify();
            return Poll::Pending;
        }

        let mut guard = this.state.lock().unwrap();
        if let Some(result) = guard.result.take() {
            drop(guard);
            match result {
                Ok(value) => Poll::Ready(value),
                Err(payload) => panic::resume_unwind(payload),
            }
        } else {
            guard.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl<F, R> Drop for TaskFuture<F, R> {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.state.lock() {
            guard.cancelled = true;
            guard.waker = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::sync::Condvar;
    use std::task::Wake;
    use std::thread;

    #[repr(C)]
    struct MSG {
        hwnd: HWND,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
        time: u32,
        pt_x: i32,
        pt_y: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn DispatchMessageW(msg: *const MSG) -> LRESULT;
        fn PeekMessageW(
            msg: *mut MSG,
            hwnd: HWND,
            msg_filter_min: u32,
            msg_filter_max: u32,
            remove_msg: u32,
        ) -> i32;
        fn TranslateMessage(msg: *const MSG) -> i32;
    }

    const PM_REMOVE: u32 = 0x0001;

    thread_local! {
        static MOCK_EVENT_QUEUE: std::cell::RefCell<Vec<*mut iTJSDispatch2>> =
            const { std::cell::RefCell::new(Vec::new()) };
    }

    unsafe extern "system" fn mock_tjs_string_from_wstr(s: *mut tTJSString, _str: *const tjs_char) {
        unsafe {
            (*s)._base.Ptr = ptr::null_mut();
        }
    }

    unsafe extern "system" fn mock_tjs_string_dtor(s: *mut tTJSString) {
        unsafe {
            (*s)._base.Ptr = ptr::null_mut();
        }
    }

    unsafe extern "system" fn mock_tvp_post_event(
        _source: *mut iTJSDispatch2,
        target: *mut iTJSDispatch2,
        _eventname: *mut ttstr,
        _tag: tjs_uint32,
        _flag: tjs_uint32,
        _numargs: tjs_uint,
        _args: *mut tTJSVariant,
    ) {
        if !target.is_null() {
            unsafe { (*target).add_ref() };
            MOCK_EVENT_QUEUE.with(|q| q.borrow_mut().push(target));
        }
    }

    unsafe extern "system" fn mock_tvp_cancel_source_events(source: *mut iTJSDispatch2) {
        MOCK_EVENT_QUEUE.with(|q| {
            let mut queue = q.borrow_mut();
            let mut i = 0;
            while i < queue.len() {
                if queue[i] == source {
                    let target = queue.remove(i);
                    unsafe { (*target).release() };
                } else {
                    i += 1;
                }
            }
        });
    }

    fn deliver_mock_events() {
        loop {
            let target = MOCK_EVENT_QUEUE.with(|q| {
                let mut queue = q.borrow_mut();
                if queue.is_empty() {
                    None
                } else {
                    Some(queue.remove(0))
                }
            });
            let Some(target) = target else {
                break;
            };
            unsafe {
                if TJSIsObjectValid((*target).is_valid(0, ptr::null(), ptr::null_mut(), target)) {
                    (*target).func_call(
                        0,
                        ptr::null(),
                        ptr::null_mut(),
                        ptr::null_mut(),
                        0,
                        ptr::null_mut(),
                        target,
                    );
                }
                (*target).release();
            }
        }
    }

    unsafe extern "C" fn mock_query_functions(
        _this: *mut iTVPFunctionExporter,
        _name: *mut *const tjs_char,
        _function: *mut *mut c_void,
        _count: tjs_uint,
    ) -> bool {
        false
    }

    unsafe extern "C" fn mock_query_functions_by_narrow_string(
        _this: *mut iTVPFunctionExporter,
        name: *mut *const i8,
        function: *mut *mut c_void,
        count: tjs_uint,
    ) -> bool {
        for i in 0..(count as usize) {
            let sym = unsafe { CStr::from_ptr(*name.add(i)).to_str().unwrap_or("") };
            let ptr: *mut c_void = match sym {
                "tTJSString::tTJSString(const tjs_char *)" => {
                    mock_tjs_string_from_wstr as *const () as *mut c_void
                }
                "tTJSString::~ tTJSString()" => mock_tjs_string_dtor as *const () as *mut c_void,
                "void ::TVPPostEvent(iTJSDispatch2 *,iTJSDispatch2 *,ttstr &,tjs_uint32,tjs_uint32,tjs_uint,tTJSVariant *)" => {
                    mock_tvp_post_event as *const () as *mut c_void
                }
                "void ::TVPCancelSourceEvents(iTJSDispatch2 *)" => {
                    mock_tvp_cancel_source_events as *const () as *mut c_void
                }
                _ => return false,
            };
            unsafe { *function.add(i) = ptr };
        }
        true
    }

    static MOCK_EXPORTER_VTABLE: iTVPFunctionExporter__bindgen_vtable =
        iTVPFunctionExporter__bindgen_vtable {
            iTVPFunctionExporter_QueryFunctions: mock_query_functions,
            iTVPFunctionExporter_QueryFunctionsByNarrowString:
                mock_query_functions_by_narrow_string,
        };

    static mut MOCK_EXPORTER: iTVPFunctionExporter = iTVPFunctionExporter {
        vtable_: &MOCK_EXPORTER_VTABLE,
    };

    struct ThreadSignal {
        ready: Mutex<bool>,
        cv: Condvar,
    }

    impl Wake for ThreadSignal {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            let mut ready = self.ready.lock().unwrap();
            *ready = true;
            self.cv.notify_one();
        }
    }

    fn block_on<F: Future>(mut fut: F) -> F::Output {
        let signal = Arc::new(ThreadSignal {
            ready: Mutex::new(false),
            cv: Condvar::new(),
        });
        let waker = Waker::from(Arc::clone(&signal));
        let mut cx = Context::from_waker(&waker);
        let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
        loop {
            if let Poll::Ready(output) = fut.as_mut().poll(&mut cx) {
                return output;
            }
            let mut ready = signal.ready.lock().unwrap();
            while !*ready {
                ready = signal.cv.wait(ready).unwrap();
            }
            *ready = false;
        }
    }

    #[test]
    fn execute_runs_closure_on_window_thread() {
        unsafe {
            TVPInitImportStub(&raw mut MOCK_EXPORTER);
        }

        let executor = AsyncExecutor::new();
        let win_tid = executor.window_thread_id();
        assert!(executor.is_window_thread());

        let done = Arc::new(AtomicBool::new(false));
        let done_bg = Arc::clone(&done);
        let exec_bg = executor.clone();

        let worker = thread::spawn(move || {
            assert!(!exec_bg.is_window_thread());

            let (ran_tid, on_win_thread, sum) = block_on(exec_bg.execute({
                let exec_inner = exec_bg.clone();
                move || {
                    let tid = unsafe { GetCurrentThreadId() };
                    (tid, exec_inner.is_window_thread(), 20 + 22)
                }
            }));
            assert_eq!(ran_tid, win_tid);
            assert!(on_win_thread);
            assert_eq!(sum, 42);

            let panic_res = panic::catch_unwind(AssertUnwindSafe(|| {
                block_on(exec_bg.execute(|| -> i32 {
                    panic!("window thread panic test");
                }))
            }));
            assert!(panic_res.is_err());

            done_bg.store(true, Ordering::Release);
        });

        while !done.load(Ordering::Acquire) {
            let mut msg: MSG = unsafe { std::mem::zeroed() };
            while unsafe { PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            deliver_mock_events();
            thread::sleep(std::time::Duration::from_millis(1));
        }

        worker.join().unwrap();
        drop(executor);
        deliver_mock_events();
    }
}
