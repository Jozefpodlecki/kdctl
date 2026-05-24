use std::io;
use std::ptr;
use flexi_logger::{DeferredNow, writers::LogWriter};
use log::Record;
use winapi::um::winbase::{RegisterEventSourceW, DeregisterEventSource, ReportEventW};
use winapi::um::winnt::{EVENTLOG_ERROR_TYPE, EVENTLOG_WARNING_TYPE, EVENTLOG_INFORMATION_TYPE};
use winapi::shared::ntdef::NULL;

pub struct EventLogWriter {
    handle: winapi::um::winnt::HANDLE,
}

unsafe impl Send for EventLogWriter {}
unsafe impl Sync for EventLogWriter {}

impl EventLogWriter {
    pub fn new(source_name: &str) -> Self {
        let wide: Vec<u16> = source_name.encode_utf16().chain(Some(0)).collect();
        let handle = unsafe { RegisterEventSourceW(ptr::null_mut(), wide.as_ptr()) };
        
        Self { handle }
    }
}

impl Drop for EventLogWriter {
    fn drop(&mut self) {
        if self.handle != NULL {
            unsafe { DeregisterEventSource(self.handle); }
        }
    }
}

impl LogWriter for EventLogWriter {
    fn write(&self, _now: &mut DeferredNow, record: &Record) -> io::Result<()> {
        if self.handle == NULL {
            return Ok(());
        }
        
        let (event_type, event_id) = match record.level() {
            log::Level::Error => (EVENTLOG_ERROR_TYPE, 1),
            log::Level::Warn => (EVENTLOG_WARNING_TYPE, 2),
            _ => (EVENTLOG_INFORMATION_TYPE, 3),
        };
        
        let msg = format!("{}", record.args());
        let wide_msg: Vec<u16> = msg.encode_utf16().chain(Some(0)).collect();
        let strings = [wide_msg.as_ptr()];
        
        unsafe {
            ReportEventW(
                self.handle,
                event_type,
                0,
                event_id,
                ptr::null_mut(),
                1,
                0,
                strings.as_ptr() as *const *const u16 as *mut *const u16,
                ptr::null_mut(),
            );
        }
        
        Ok(())
    }
    
    fn flush(&self) -> io::Result<()> {
        Ok(())
    }
    
    fn max_log_level(&self) -> log::LevelFilter {
        log::LevelFilter::Info
    }
}