use krkrz_plugin_base::param::TypeError;
use krkrz_plugin_base::{tp_stub::*, *};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Controls::{
    TASKDIALOG_BUTTON, TASKDIALOGCONFIG, TDCBF_ABORT_BUTTON, TDCBF_CANCEL_BUTTON,
    TDCBF_CLOSE_BUTTON, TDCBF_CONTINUE_BUTTON, TDCBF_HELP_BUTTON, TDCBF_IGNORE_BUTTON,
    TDCBF_NO_BUTTON, TDCBF_OK_BUTTON, TDCBF_RETRY_BUTTON, TDCBF_TRYAGAIN_BUTTON, TDCBF_YES_BUTTON,
    TDF_ALLOW_DIALOG_CANCELLATION, TDF_CALLBACK_TIMER, TDF_CAN_BE_MINIMIZED, TDF_ENABLE_HYPERLINKS,
    TDF_EXPAND_FOOTER_AREA, TDF_EXPANDED_BY_DEFAULT, TDF_NO_DEFAULT_RADIO_BUTTON,
    TDF_NO_SET_FOREGROUND, TDF_POSITION_RELATIVE_TO_WINDOW, TDF_RTL_LAYOUT,
    TDF_SHOW_MARQUEE_PROGRESS_BAR, TDF_SHOW_PROGRESS_BAR, TDF_SIZE_TO_CONTENT,
    TDF_USE_COMMAND_LINKS, TDF_USE_COMMAND_LINKS_NO_ICON, TDF_USE_HICON_FOOTER, TDF_USE_HICON_MAIN,
    TDF_VERIFICATION_FLAG_CHECKED, TaskDialogIndirect,
};
use windows::Win32::UI::WindowsAndMessaging::{
    IDABORT, IDCANCEL, IDCLOSE, IDCONTINUE, IDHELP, IDIGNORE, IDNO, IDOK, IDRETRY, IDTRYAGAIN,
    IDYES,
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
    exinfo: Option<ttstr>,
    exctlt: Option<ttstr>,
    colctlt: Option<ttstr>,
    footer: Option<ttstr>,
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
            exinfo: None,
            exctlt: None,
            colctlt: None,
            footer: None,
        }
    }

    #[tjs(case = constant, static_member)]
    fn button_abort() -> tTJSVariant {
        tTJSVariant::from(TDCBF_ABORT_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_cancel() -> tTJSVariant {
        tTJSVariant::from(TDCBF_CANCEL_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_close() -> tTJSVariant {
        tTJSVariant::from(TDCBF_CLOSE_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_continue() -> tTJSVariant {
        tTJSVariant::from(TDCBF_CONTINUE_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_help() -> tTJSVariant {
        tTJSVariant::from(TDCBF_HELP_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_ignore() -> tTJSVariant {
        tTJSVariant::from(TDCBF_IGNORE_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_no() -> tTJSVariant {
        tTJSVariant::from(TDCBF_NO_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_ok() -> tTJSVariant {
        tTJSVariant::from(TDCBF_OK_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_retry() -> tTJSVariant {
        tTJSVariant::from(TDCBF_RETRY_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_tryagain() -> tTJSVariant {
        tTJSVariant::from(TDCBF_TRYAGAIN_BUTTON.0)
    }

    #[tjs(case = constant, static_member)]
    fn button_yes() -> tTJSVariant {
        tTJSVariant::from(TDCBF_YES_BUTTON.0)
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

    #[tjs(case = constant, static_member)]
    fn idabort() -> tTJSVariant {
        tTJSVariant::from(IDABORT.0)
    }

    #[tjs(case = constant, static_member)]
    fn idcancel() -> tTJSVariant {
        tTJSVariant::from(IDCANCEL.0)
    }

    #[tjs(case = constant, static_member)]
    fn idclose() -> tTJSVariant {
        tTJSVariant::from(IDCLOSE.0)
    }

    #[tjs(case = constant, static_member)]
    fn idcontinue() -> tTJSVariant {
        tTJSVariant::from(IDCONTINUE.0)
    }

    #[tjs(case = constant, static_member)]
    fn idhelp() -> tTJSVariant {
        tTJSVariant::from(IDHELP.0)
    }

    #[tjs(case = constant, static_member)]
    fn idignore() -> tTJSVariant {
        tTJSVariant::from(IDIGNORE.0)
    }

    #[tjs(case = constant, static_member)]
    fn idno() -> tTJSVariant {
        tTJSVariant::from(IDNO.0)
    }

    #[tjs(case = constant, static_member)]
    fn idok() -> tTJSVariant {
        tTJSVariant::from(IDOK.0)
    }

    #[tjs(case = constant, static_member)]
    fn idretry() -> tTJSVariant {
        tTJSVariant::from(IDRETRY.0)
    }

    #[tjs(case = constant, static_member)]
    fn idtryagain() -> tTJSVariant {
        tTJSVariant::from(IDTRYAGAIN.0)
    }

    #[tjs(case = constant, static_member)]
    fn idyes() -> tTJSVariant {
        tTJSVariant::from(IDYES.0)
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
    fn set_default_button(&mut self, btn: i64) {
        self.cfg.nDefaultButton = btn as i32;
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

    #[tjs(case = camel, return_this, method)]
    fn set_expanded_information(&mut self, info: ttstr) {
        self.cfg.pszExpandedInformation = PCWSTR::from_raw(info.c_str());
        self.exinfo = Some(info);
    }

    #[tjs(case = camel, return_this, method)]
    fn set_expanded_control_text(&mut self, text: ttstr) {
        self.cfg.pszExpandedControlText = PCWSTR::from_raw(text.c_str());
        self.exctlt = Some(text);
    }

    #[tjs(case = camel, return_this, method)]
    fn set_collapsed_control_text(&mut self, text: ttstr) {
        self.cfg.pszCollapsedControlText = PCWSTR::from_raw(text.c_str());
        self.colctlt = Some(text);
    }

    #[tjs(case = camel, return_this, method)]
    fn set_footer_icon(&mut self, icon: &mut tTJSVariant) -> Result<(), TypeError> {
        if icon.is_integer() {
            let icon = i64::to_param(icon)?;
            self.cfg.Anonymous2.pszFooterIcon = PCWSTR(icon as u16 as *const _);
        } else {
            let s = ttstr::to_param(icon)?;
            self.cfg.Anonymous2.pszFooterIcon = PCWSTR::from_raw(s.c_str());
            self.icon = Some(s);
        }
        Ok(())
    }

    #[tjs(case = camel, return_this, method)]
    fn set_footer(&mut self, text: ttstr) {
        self.cfg.pszFooter = PCWSTR::from_raw(text.c_str());
        self.footer = Some(text);
    }

    #[tjs(case = camel, return_this, method)]
    fn set_width(&mut self, width: i64) {
        self.cfg.cxWidth = width as u32;
    }

    #[tjs(case = camel, return_this)]
    fn use_command_links(&mut self) {
        self.cfg.dwFlags.0 |= TDF_USE_COMMAND_LINKS.0;
    }

    #[tjs(case = camel, return_this)]
    fn allow_dialog_cancellation(&mut self) {
        self.cfg.dwFlags.0 |= TDF_ALLOW_DIALOG_CANCELLATION.0;
    }

    #[tjs(case = camel, return_this)]
    fn callback_timer(&mut self) {
        self.cfg.dwFlags.0 |= TDF_CALLBACK_TIMER.0;
    }

    #[tjs(case = camel, return_this)]
    fn can_be_minimized(&mut self) {
        self.cfg.dwFlags.0 |= TDF_CAN_BE_MINIMIZED.0;
    }

    #[tjs(case = camel, return_this)]
    fn enable_hyperlinks(&mut self) {
        self.cfg.dwFlags.0 |= TDF_ENABLE_HYPERLINKS.0;
    }

    #[tjs(case = camel, return_this)]
    fn expanded_by_default(&mut self) {
        self.cfg.dwFlags.0 |= TDF_EXPANDED_BY_DEFAULT.0;
    }

    #[tjs(case = camel, return_this)]
    fn expand_footer_area(&mut self) {
        self.cfg.dwFlags.0 |= TDF_EXPAND_FOOTER_AREA.0;
    }

    #[tjs(case = camel, return_this)]
    fn no_default_radio_button(&mut self) {
        self.cfg.dwFlags.0 |= TDF_NO_DEFAULT_RADIO_BUTTON.0;
    }

    #[tjs(case = camel, return_this)]
    fn no_set_foreground(&mut self) {
        self.cfg.dwFlags.0 |= TDF_NO_SET_FOREGROUND.0;
    }

    #[tjs(case = camel, return_this)]
    fn position_relative_to_window(&mut self) {
        self.cfg.dwFlags.0 |= TDF_POSITION_RELATIVE_TO_WINDOW.0;
    }

    #[tjs(case = camel, return_this)]
    fn rtl_layout(&mut self) {
        self.cfg.dwFlags.0 |= TDF_RTL_LAYOUT.0;
    }

    #[tjs(case = camel, return_this)]
    fn show_marquee_progress_bar(&mut self) {
        self.cfg.dwFlags.0 |= TDF_SHOW_MARQUEE_PROGRESS_BAR.0;
    }

    #[tjs(case = camel, return_this)]
    fn show_progress_bar(&mut self) {
        self.cfg.dwFlags.0 |= TDF_SHOW_PROGRESS_BAR.0;
    }

    #[tjs(case = camel, return_this)]
    fn size_to_content(&mut self) {
        self.cfg.dwFlags.0 |= TDF_SIZE_TO_CONTENT.0;
    }

    #[tjs(case = camel, return_this)]
    fn use_command_links_no_icon(&mut self) {
        self.cfg.dwFlags.0 |= TDF_USE_COMMAND_LINKS_NO_ICON.0;
    }

    #[tjs(case = camel, return_this)]
    fn use_hicon_footer(&mut self) {
        self.cfg.dwFlags.0 |= TDF_USE_HICON_FOOTER.0;
    }

    #[tjs(case = camel, return_this)]
    fn use_hicon_main(&mut self) {
        self.cfg.dwFlags.0 |= TDF_USE_HICON_MAIN.0;
    }

    #[tjs(case = camel, return_this)]
    fn verification_flag_checked(&mut self) {
        self.cfg.dwFlags.0 |= TDF_VERIFICATION_FLAG_CHECKED.0;
    }

    #[tjs(case = camel, return_this)]
    fn add_common_button(&mut self, flags: i64) {
        self.cfg.dwCommonButtons.0 = flags as i32;
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
