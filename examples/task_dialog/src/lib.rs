use krkrz_plugin_base::param::TypeError;
use krkrz_plugin_base::{tp_stub::*, *};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Controls::{
    TASKDIALOG_BUTTON, TASKDIALOGCONFIG, TDF_USE_COMMAND_LINKS, TaskDialogIndirect,
};
use windows::core::PCWSTR;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(task_dialog);

struct TaskDialog {
    cfg: TASKDIALOGCONFIG,
    title: Option<ttstr>,
    instr: Option<ttstr>,
    content: Option<ttstr>,
    icon: Option<ttstr>,
    obtns: Vec<ttstr>,
    btns: Vec<TASKDIALOG_BUTTON>,
}

#[Tjs2Class]
impl TaskDialog {
    fn new() -> Self {
        let mut cfg = TASKDIALOGCONFIG::default();
        cfg.cbSize = size_of::<TASKDIALOGCONFIG>() as u32;
        Self {
            cfg,
            title: None,
            instr: None,
            content: None,
            icon: None,
            obtns: Vec::new(),
            btns: Vec::new(),
        }
    }

    #[tjs(case = constant, static_member)]
    fn icon_warning() -> tTJSVariant {
        tTJSVariant::from(65535i64)
    }

    #[tjs(case = constant, static_member)]
    fn icon_error() -> tTJSVariant {
        tTJSVariant::from(65534i64)
    }

    #[tjs(case = constant, static_member)]
    fn icon_information() -> tTJSVariant {
        tTJSVariant::from(65533i64)
    }

    #[tjs(case = camel, return_this, method)]
    fn set_title(&mut self, title: ttstr) {
        self.cfg.pszWindowTitle = PCWSTR::from_raw(title.c_str());
        self.title = Some(title);
    }

    #[tjs(case = camel, return_this, method)]
    fn set_main_instruction(&mut self, instr: ttstr) {
        self.cfg.pszMainInstruction = PCWSTR::from_raw(instr.c_str());
        self.instr = Some(instr);
    }

    #[tjs(case = camel, return_this, method)]
    fn set_content(&mut self, content: ttstr) {
        self.cfg.pszContent = PCWSTR::from_raw(content.c_str());
        self.content = Some(content);
    }

    #[tjs(case = camel, return_this, method)]
    fn set_icon(&mut self, icon: &mut tTJSVariant) -> Result<(), TypeError> {
        if icon.is_integer() {
            let icon = i64::to_param(icon)?;
            self.cfg.Anonymous1.pszMainIcon = PCWSTR(icon as u16 as *const _);
        } else {
            let s = ttstr::to_param(icon)?;
            self.cfg.Anonymous1.pszMainIcon = PCWSTR::from_raw(s.c_str());
            self.icon = Some(s);
        }
        Ok(())
    }

    #[tjs(case = camel, return_this)]
    fn use_command_links(&mut self) {
        self.cfg.dwFlags.0 |= TDF_USE_COMMAND_LINKS.0;
    }

    #[tjs(case = camel, return_this)]
    fn add_button(&mut self, id: i64, mut btn: ttstr, info: Option<ttstr>) {
        if let Some(info) = info {
            btn += "\n";
            btn += &info;
        }
        self.btns.push(TASKDIALOG_BUTTON {
            nButtonID: id as i32,
            pszButtonText: PCWSTR::from_raw(btn.c_str()),
        });
        self.obtns.push(btn);
    }

    fn show(&mut self) -> Result<i64, std::io::Error> {
        self.cfg.cButtons = self.btns.len() as u32;
        self.cfg.pButtons = self.btns.as_ptr();
        let hwnd = unsafe { TVPGetApplicationWindowHandle() };
        if hwnd.addr() != usize::MAX {
            self.cfg.hwndParent = HWND(hwnd as *mut _);
        }
        let mut clicked_button: i32 = 0;
        unsafe { TaskDialogIndirect(&self.cfg, Some(&mut clicked_button), None, None) }
            .map_err(|x| std::io::Error::from_raw_os_error(x.code().0))?;
        Ok(clicked_button as i64)
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let task_dialog = TaskDialog::create_native_class().1;
    register_var!(task_dialog);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[task-dialog]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(task_dialog);
    unsafe { TVPUninitImportStub() };
    0
}
