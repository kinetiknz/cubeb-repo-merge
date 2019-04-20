use super::utils::{
    test_change_default_device, test_create_device_change_listener, test_get_default_device,
    test_get_devices_in_scope, test_ops_stream_operation, Scope, TestDevicePlugger,
};
use super::*;
use std::fmt::Debug;

#[ignore]
#[test]
fn test_switch_device() {
    test_switch_device_in_scope(Scope::Input);
    test_switch_device_in_scope(Scope::Output);
}

fn test_switch_device_in_scope(scope: Scope) {
    use std::thread;
    use std::time::Duration;

    // Do nothing if there is no 2 available devices at least.
    let devices = test_get_devices_in_scope(scope.clone());
    if devices.len() < 2 {
        println!("Need 2 devices for {:?} at least.", scope);
        return;
    }

    println!(
        "Switch default device for {:?} while the stream is working.",
        scope
    );

    let count = Arc::new(Mutex::new(0));
    let also_count = Arc::clone(&count);
    let listener = test_create_device_change_listener(scope.clone(), move |_addresses| {
        let mut cnt = also_count.lock().unwrap();
        *cnt += 1;
        NO_ERR
    });
    listener.start();

    let mut changed_watcher = Watcher::new(&count);
    test_get_started_stream_in_scope(scope.clone(), move |_stream| loop {
        thread::sleep(Duration::from_millis(500));
        changed_watcher.prepare();
        assert!(test_change_default_device(scope.clone()).unwrap());
        changed_watcher.wait_for_change();
        if changed_watcher.current_result() >= devices.len() {
            break;
        }
    });
}

fn test_get_started_stream_in_scope<F>(scope: Scope, operation: F)
where
    F: FnOnce(*mut ffi::cubeb_stream),
{
    use std::f32::consts::PI;
    const SAMPLE_FREQUENCY: u32 = 48_000;

    // Make sure the parameters meet the requirements of AudioUnitContext::stream_init
    // (in the comments).
    let mut stream_params = ffi::cubeb_stream_params::default();
    stream_params.format = ffi::CUBEB_SAMPLE_S16NE;
    stream_params.rate = SAMPLE_FREQUENCY;
    stream_params.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream_params.channels = 1;
    stream_params.layout = ffi::CUBEB_LAYOUT_MONO;

    let (input_params, output_params) = match scope {
        Scope::Input => (
            &mut stream_params as *mut ffi::cubeb_stream_params,
            ptr::null_mut(),
        ),
        Scope::Output => (
            ptr::null_mut(),
            &mut stream_params as *mut ffi::cubeb_stream_params,
        ),
    };

    extern "C" fn state_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        state: ffi::cubeb_state,
    ) {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
        assert_ne!(state, ffi::CUBEB_STATE_ERROR);
    }

    extern "C" fn input_data_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        input_buffer: *const c_void,
        output_buffer: *mut c_void,
        nframes: i64,
    ) -> i64 {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
        assert!(!input_buffer.is_null());
        assert!(output_buffer.is_null());
        nframes
    }

    let mut position: i64 = 0; // TODO: Use Atomic instead.

    fn f32_to_i16_sample(x: f32) -> i16 {
        (x * f32::from(i16::max_value())) as i16
    }

    extern "C" fn output_data_callback(
        stream: *mut ffi::cubeb_stream,
        user_ptr: *mut c_void,
        input_buffer: *const c_void,
        output_buffer: *mut c_void,
        nframes: i64,
    ) -> i64 {
        assert!(!stream.is_null());
        assert!(!user_ptr.is_null());
        assert!(input_buffer.is_null());
        assert!(!output_buffer.is_null());

        let buffer = unsafe {
            let ptr = output_buffer as *mut i16;
            let len = nframes as usize;
            slice::from_raw_parts_mut(ptr, len)
        };

        let position = unsafe { &mut *(user_ptr as *mut i64) };

        // Generate tone on the fly.
        for data in buffer.iter_mut() {
            let t1 = (2.0 * PI * 350.0 * (*position) as f32 / SAMPLE_FREQUENCY as f32).sin();
            let t2 = (2.0 * PI * 440.0 * (*position) as f32 / SAMPLE_FREQUENCY as f32).sin();
            *data = f32_to_i16_sample(0.5 * (t1 + t2));
            *position += 1;
        }

        nframes
    }

    test_ops_stream_operation(
        "stream",
        ptr::null_mut(), // Use default input device.
        input_params,
        ptr::null_mut(), // Use default output device.
        output_params,
        4096, // TODO: Get latency by get_min_latency instead ?
        match scope {
            Scope::Input => Some(input_data_callback),
            Scope::Output => Some(output_data_callback),
        },
        Some(state_callback),
        &mut position as *mut i64 as *mut c_void,
        |stream| {
            assert_eq!(unsafe { OPS.stream_start.unwrap()(stream) }, ffi::CUBEB_OK);
            operation(stream);
            assert_eq!(unsafe { OPS.stream_stop.unwrap()(stream) }, ffi::CUBEB_OK);
        },
    );
}

#[ignore]
#[test]
fn test_plug_and_unplug_device() {
    println!("NOTICE: The test will hang if the default input or output is an aggregate device.\nWe will fix this later.");
    let has_input = test_get_default_device(Scope::Input).is_some();
    let has_output = test_get_default_device(Scope::Output).is_some();

    // Initialize the the mutex (whose type is OwnedCriticalSection) within AudioUnitContext,
    // by AudioUnitContext::Init, to make the mutex work.
    let mut context = AudioUnitContext::new();
    context.init();

    // Register the devices-changed callbacks.
    let input_count = Arc::new(Mutex::new(0u32));
    let also_input_count = Arc::clone(&input_count);
    let input_mtx_ptr = also_input_count.as_ref() as *const Mutex<u32>;

    assert!(context
        .register_device_collection_changed(
            DeviceType::INPUT,
            Some(input_changed_callback),
            input_mtx_ptr as *mut c_void,
        )
        .is_ok());

    let output_count = Arc::new(Mutex::new(0u32));
    let also_output_count = Arc::clone(&output_count);
    let output_mtx_ptr = also_output_count.as_ref() as *const Mutex<u32>;

    assert!(context
        .register_device_collection_changed(
            DeviceType::OUTPUT,
            Some(output_changed_callback),
            output_mtx_ptr as *mut c_void,
        )
        .is_ok());

    let mut input_plugger = TestDevicePlugger::new(Scope::Input).unwrap();
    let mut output_plugger = TestDevicePlugger::new(Scope::Output).unwrap();

    let mut input_watcher = Watcher::new(&input_count);
    let mut output_watcher = Watcher::new(&output_count);

    // Simulate adding devices and monitor the devices-changed callbacks.
    input_watcher.prepare();
    output_watcher.prepare();
    assert_eq!(has_input, input_plugger.plug().is_ok());
    assert_eq!(has_output, output_plugger.plug().is_ok());

    if has_input {
        input_watcher.wait_for_change();
    }
    if has_output {
        output_watcher.wait_for_change();
    }

    check_result(has_input, (1, 0), &input_watcher);
    check_result(has_output, (1, 0), &output_watcher);

    // Simulate removing devices and monitor the devices-changed callbacks.
    input_watcher.prepare();
    output_watcher.prepare();
    assert!(!has_input || input_plugger.unplug().is_ok());
    assert!(!has_output || output_plugger.unplug().is_ok());

    if has_input {
        input_watcher.wait_for_change();
    }
    if has_output {
        output_watcher.wait_for_change();
    }

    check_result(has_input, (2, 0), &input_watcher);
    check_result(has_output, (2, 0), &output_watcher);

    // The devices-changed callbacks will be unregistered when AudioUnitContext is dropped.

    // Helpers for this test.
    fn check_result<T: Clone + PartialEq + Debug>(
        has_device: bool,
        expected: (T, T),
        watcher: &Watcher<T>,
    ) {
        let expected_result = if has_device { expected.0 } else { expected.1 };
        assert_eq!(watcher.current_result(), expected_result);
    }

    extern "C" fn input_changed_callback(context: *mut ffi::cubeb, data: *mut c_void) {
        println!(
            "Input device collection @ {:p} is changed. Data @ {:p}",
            context, data
        );
        let count = unsafe { &*(data as *const Mutex<i32>) };
        {
            let mut guard = count.lock().unwrap();
            *guard += 1;
        }
    }

    extern "C" fn output_changed_callback(context: *mut ffi::cubeb, data: *mut c_void) {
        println!(
            "output device collection @ {:p} is changed. Data @ {:p}",
            context, data
        );
        let count = unsafe { &*(data as *const Mutex<i32>) };
        {
            let mut guard = count.lock().unwrap();
            *guard += 1;
        }
    }
}

#[ignore]
#[test]
fn test_register_device_changed_callback_to_check_default_device_changed() {
    let input_devices = test_get_devices_in_scope(Scope::Input).len();
    let output_devices = test_get_devices_in_scope(Scope::Output).len();
    if input_devices < 2 && output_devices < 2 {
        println!("Need 2 input or 2 output devices at least.");
        return;
    }

    println!("NOTICE: The test will hang if the default input or output is an aggregate device.\nWe will fix this later.");

    let changed_count = Arc::new(Mutex::new(0u32));
    let also_changed_count = Arc::clone(&changed_count);
    let mtx_ptr = also_changed_count.as_ref() as *const Mutex<u32>;

    let input_count = high_pass_filter(input_devices as u32, 2);
    let output_count = high_pass_filter(output_devices as u32, 2);

    test_get_duplex_stream_with_device_changed_callback(
        "stream: test callback for default device changed",
        ptr::null_mut(), // Use default input device.
        ptr::null_mut(), // Use default output device.
        mtx_ptr as *mut c_void,
        callback,
        |stream| {
            let mut changed_watcher = Watcher::new(&changed_count);

            for _ in 0..input_count {
                // While the stream is re-initializing for the default device switch,
                // switching for the default device again will be ignored.
                while stream.switching_device.load(atomic::Ordering::SeqCst) {}
                changed_watcher.prepare();
                assert!(test_change_default_device(Scope::Input).unwrap());
                changed_watcher.wait_for_change();
            }

            for _ in 0..output_count {
                // While the stream is re-initializing for the default device switch,
                // switching for the default device again will be ignored.
                while stream.switching_device.load(atomic::Ordering::SeqCst) {}
                changed_watcher.prepare();
                assert!(test_change_default_device(Scope::Output).unwrap());
                changed_watcher.wait_for_change();
            }
        },
    );

    extern "C" fn callback(data: *mut c_void) {
        println!("Device change callback. data @ {:p}", data);
        let count = unsafe { &*(data as *const Mutex<i32>) };
        {
            let mut guard = count.lock().unwrap();
            *guard += 1;
        }
    }

    fn high_pass_filter(value: u32, limit: u32) -> u32 {
        if value < limit {
            0
        } else {
            value
        }
    }
}

#[ignore]
#[test]
fn test_register_device_changed_callback_to_check_input_alive_changed() {
    let has_input = test_get_default_device(Scope::Input).is_some();
    if !has_input {
        println!("Need one input device at least.");
        return;
    }

    let changed_count = Arc::new(Mutex::new(0u32));
    let also_changed_count = Arc::clone(&changed_count);
    let mtx_ptr = also_changed_count.as_ref() as *const Mutex<u32>;

    let mut input_plugger = TestDevicePlugger::new(Scope::Input).unwrap();

    assert_eq!(has_input, input_plugger.plug().is_ok());

    test_get_duplex_stream_with_device_changed_callback(
        "stream: test callback for input alive changed",
        input_plugger.get_device_id() as ffi::cubeb_devid,
        ptr::null_mut(), // Use default output device.
        mtx_ptr as *mut c_void,
        callback,
        |_stream| {
            let mut changed_watcher = Watcher::new(&changed_count);
            changed_watcher.prepare();
            assert!(input_plugger.unplug().is_ok());
            changed_watcher.wait_for_change();
        },
    );

    extern "C" fn callback(data: *mut c_void) {
        println!("Device change callback. data @ {:p}", data);
        let count = unsafe { &*(data as *const Mutex<i32>) };
        {
            let mut guard = count.lock().unwrap();
            *guard += 1;
        }
    }
}

fn test_get_duplex_stream_with_device_changed_callback<F>(
    name: &'static str,
    input_device: ffi::cubeb_devid,
    output_device: ffi::cubeb_devid,
    data: *mut c_void,
    callback: extern "C" fn(*mut c_void),
    operation: F,
) where
    F: FnOnce(&mut AudioUnitStream),
{
    test_ops_duplex_stream_operation(name, input_device, output_device, data, |stream| {
        let stm = unsafe { &mut *(stream as *mut AudioUnitStream) };
        assert!(stm.register_device_changed_callback(Some(callback)).is_ok());
        operation(stm);
        assert!(stm.register_device_changed_callback(None).is_ok());
    });
}

fn test_ops_duplex_stream_operation<F>(
    name: &'static str,
    input_device: ffi::cubeb_devid,
    output_device: ffi::cubeb_devid,
    data: *mut c_void,
    operation: F,
) where
    F: FnOnce(*mut ffi::cubeb_stream),
{
    // Make sure the parameters meet the requirements of AudioUnitContext::stream_init
    // (in the comments).
    let mut output_params = ffi::cubeb_stream_params::default();
    output_params.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    output_params.rate = 44100;
    output_params.channels = 2;
    output_params.layout = ffi::CUBEB_LAYOUT_STEREO;
    output_params.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    let mut input_params = ffi::cubeb_stream_params::default();
    input_params.format = ffi::CUBEB_SAMPLE_S16NE;
    input_params.rate = 48000;
    input_params.channels = 1;
    input_params.layout = ffi::CUBEB_LAYOUT_MONO;
    input_params.prefs = ffi::CUBEB_STREAM_PREF_NONE;

    test_ops_stream_operation(
        name,
        input_device,
        &mut input_params,
        output_device,
        &mut output_params,
        4096, // TODO: Get latency by get_min_latency instead ?
        None, // No data callback.
        None, // No state callback.
        data,
        operation,
    );
}

struct Watcher<T: Clone + PartialEq> {
    watching: Arc<Mutex<T>>,
    current: Option<T>,
}

impl<T: Clone + PartialEq> Watcher<T> {
    fn new(value: &Arc<Mutex<T>>) -> Self {
        Self {
            watching: Arc::clone(value),
            current: None,
        }
    }

    fn prepare(&mut self) {
        self.current = Some(self.current_result());
    }

    fn wait_for_change(&self) {
        loop {
            if self.current_result() != self.current.clone().unwrap() {
                break;
            }
        }
    }

    fn current_result(&self) -> T {
        let guard = self.watching.lock().unwrap();
        guard.clone()
    }
}
