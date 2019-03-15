use super::utils::{
    test_get_default_device, test_get_empty_stream, test_get_locked_context, Scope,
};
use super::*;

// make_sized_audio_channel_layout
// ------------------------------------
#[test]
fn test_make_sized_audio_channel_layout() {
    for channels in 1..10 {
        let size = mem::size_of::<AudioChannelLayout>()
            + (channels - 1) * mem::size_of::<AudioChannelDescription>();
        let _ = make_sized_audio_channel_layout(size);
    }
}

#[test]
#[should_panic]
fn test_make_sized_audio_channel_layout_with_wrong_size() {
    // let _ = make_sized_audio_channel_layout(0);
    let one_channel_size = mem::size_of::<AudioChannelLayout>();
    let padding_size = 10;
    assert_ne!(mem::size_of::<AudioChannelDescription>(), padding_size);
    let wrong_size = one_channel_size + padding_size;
    let _ = make_sized_audio_channel_layout(wrong_size);
}

// to_string
// ------------------------------------
#[test]
fn test_to_string() {
    assert_eq!(to_string(&io_side::INPUT), "input");
    assert_eq!(to_string(&io_side::OUTPUT), "output");
}

// has_input
// ------------------------------------
// TODO

// has_output
// ------------------------------------
// TODO

// channel_label_to_cubeb_channel
// ------------------------------------
#[test]
fn test_channel_label_to_cubeb_channel() {
    let pairs = [
        (kAudioChannelLabel_Left, ChannelLayout::FRONT_LEFT),
        (kAudioChannelLabel_Right, ChannelLayout::FRONT_RIGHT),
        (kAudioChannelLabel_Center, ChannelLayout::FRONT_CENTER),
        (kAudioChannelLabel_LFEScreen, ChannelLayout::LOW_FREQUENCY),
        (kAudioChannelLabel_LeftSurround, ChannelLayout::BACK_LEFT),
        (kAudioChannelLabel_RightSurround, ChannelLayout::BACK_RIGHT),
        (
            kAudioChannelLabel_LeftCenter,
            ChannelLayout::FRONT_LEFT_OF_CENTER,
        ),
        (
            kAudioChannelLabel_RightCenter,
            ChannelLayout::FRONT_RIGHT_OF_CENTER,
        ),
        (
            kAudioChannelLabel_CenterSurround,
            ChannelLayout::BACK_CENTER,
        ),
        (
            kAudioChannelLabel_LeftSurroundDirect,
            ChannelLayout::SIDE_LEFT,
        ),
        (
            kAudioChannelLabel_RightSurroundDirect,
            ChannelLayout::SIDE_RIGHT,
        ),
        (
            kAudioChannelLabel_TopCenterSurround,
            ChannelLayout::TOP_CENTER,
        ),
        (
            kAudioChannelLabel_VerticalHeightLeft,
            ChannelLayout::TOP_FRONT_LEFT,
        ),
        (
            kAudioChannelLabel_VerticalHeightCenter,
            ChannelLayout::TOP_FRONT_CENTER,
        ),
        (
            kAudioChannelLabel_VerticalHeightRight,
            ChannelLayout::TOP_FRONT_RIGHT,
        ),
        (kAudioChannelLabel_TopBackLeft, ChannelLayout::TOP_BACK_LEFT),
        (
            kAudioChannelLabel_TopBackCenter,
            ChannelLayout::TOP_BACK_CENTER,
        ),
        (
            kAudioChannelLabel_TopBackRight,
            ChannelLayout::TOP_BACK_RIGHT,
        ),
        (kAudioChannelLabel_Unknown, ChannelLayout::UNDEFINED),
    ];

    for (label, channel) in pairs.iter() {
        assert_eq!(channel_label_to_cubeb_channel(*label), *channel);
    }
}

// cubeb_channel_to_channel_label
// ------------------------------------
#[test]
fn test_cubeb_channel_to_channel_label() {
    let pairs = [
        (ChannelLayout::FRONT_LEFT, kAudioChannelLabel_Left),
        (ChannelLayout::FRONT_RIGHT, kAudioChannelLabel_Right),
        (ChannelLayout::FRONT_CENTER, kAudioChannelLabel_Center),
        (ChannelLayout::LOW_FREQUENCY, kAudioChannelLabel_LFEScreen),
        (ChannelLayout::BACK_LEFT, kAudioChannelLabel_LeftSurround),
        (ChannelLayout::BACK_RIGHT, kAudioChannelLabel_RightSurround),
        (
            ChannelLayout::FRONT_LEFT_OF_CENTER,
            kAudioChannelLabel_LeftCenter,
        ),
        (
            ChannelLayout::FRONT_RIGHT_OF_CENTER,
            kAudioChannelLabel_RightCenter,
        ),
        (
            ChannelLayout::BACK_CENTER,
            kAudioChannelLabel_CenterSurround,
        ),
        (
            ChannelLayout::SIDE_LEFT,
            kAudioChannelLabel_LeftSurroundDirect,
        ),
        (
            ChannelLayout::SIDE_RIGHT,
            kAudioChannelLabel_RightSurroundDirect,
        ),
        (
            ChannelLayout::TOP_CENTER,
            kAudioChannelLabel_TopCenterSurround,
        ),
        (
            ChannelLayout::TOP_FRONT_LEFT,
            kAudioChannelLabel_VerticalHeightLeft,
        ),
        (
            ChannelLayout::TOP_FRONT_CENTER,
            kAudioChannelLabel_VerticalHeightCenter,
        ),
        (
            ChannelLayout::TOP_FRONT_RIGHT,
            kAudioChannelLabel_VerticalHeightRight,
        ),
        (ChannelLayout::TOP_BACK_LEFT, kAudioChannelLabel_TopBackLeft),
        (
            ChannelLayout::TOP_BACK_CENTER,
            kAudioChannelLabel_TopBackCenter,
        ),
        (
            ChannelLayout::TOP_BACK_RIGHT,
            kAudioChannelLabel_TopBackRight,
        ),
    ];

    for (channel, label) in pairs.iter() {
        assert_eq!(cubeb_channel_to_channel_label(*channel), *label);
    }
}

#[test]
#[should_panic]
fn test_cubeb_channel_to_channel_label_with_invalid_channel() {
    assert_eq!(
        cubeb_channel_to_channel_label(ChannelLayout::_3F4_LFE),
        kAudioChannelLabel_Unknown
    );
}

#[test]
#[should_panic]
fn test_cubeb_channel_to_channel_label_with_unknown_channel() {
    assert_eq!(
        ChannelLayout::from(ffi::CHANNEL_UNKNOWN),
        ChannelLayout::UNDEFINED
    );
    assert_eq!(
        cubeb_channel_to_channel_label(ChannelLayout::UNDEFINED),
        kAudioChannelLabel_Unknown
    );
}

// increment_active_streams
// decrement_active_streams
// active_streams
// ------------------------------------
#[test]
fn test_increase_and_decrease_active_streams() {
    test_get_locked_context(|context| {
        assert_eq!(context.active_streams, 0);

        for i in 1..10 {
            audiounit_increment_active_streams(context);
            assert_eq!(context.active_streams, i);
            assert_eq!(audiounit_active_streams(context), i);
        }

        for i in (0..9).rev() {
            audiounit_decrement_active_streams(context);
            assert_eq!(context.active_streams, i);
            assert_eq!(audiounit_active_streams(context), i);
        }
    });
}

// set_global_latency
// ------------------------------------
#[test]
fn test_set_global_latency() {
    test_get_locked_context(|context| {
        assert_eq!(context.active_streams, 0);
        audiounit_increment_active_streams(context);
        assert_eq!(context.active_streams, 1);

        for i in 0..10 {
            audiounit_set_global_latency(context, i);
            assert_eq!(context.global_latency_frames, i);
        }
    });
}

// make_silent
// ------------------------------------
#[test]
fn test_make_silent() {
    let mut array = allocate_array::<u32>(10);
    for data in array.iter_mut() {
        *data = 0xFFFF;
    }

    let mut buffer = AudioBuffer::default();
    buffer.mData = array.as_mut_ptr() as *mut c_void;
    buffer.mDataByteSize = (array.len() * mem::size_of::<u32>()) as u32;
    buffer.mNumberChannels = 1;

    audiounit_make_silent(&mut buffer);
    for data in array {
        assert_eq!(data, 0);
    }
}

// render_input
// ------------------------------------
// TODO

// input_callback
// ------------------------------------
// TODO

// mix_output_buffer
// ------------------------------------
// TODO

// minimum_resampling_input_frames
// ------------------------------------
#[test]
fn test_minimum_resampling_input_frames() {
    test_get_empty_stream(|stream| {
        // Set input and output rates to 48000 and 44100 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (48000_f64, 44100_f64));
        let frames: i64 = 100;
        let times = stream.input_hw_rate / f64::from(stream.output_stream_params.rate());
        let expected = (frames as f64 * times).ceil() as i64;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), expected);
    });
}

#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_input_rate() {
    test_get_empty_stream(|stream| {
        // Set input and output rates to 0 and 44100 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (0_f64, 44100_f64));
        let frames: i64 = 100;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), 0);
    });
}

#[test]
#[should_panic]
fn test_minimum_resampling_input_frames_zero_output_rate() {
    test_get_empty_stream(|stream| {
        // Set input and output rates to 48000 and 0 respectively.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (48000_f64, 0_f64));
        let frames: i64 = 100;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), 0);
    });
}

#[test]
fn test_minimum_resampling_input_frames_equal_input_output_rate() {
    test_get_empty_stream(|stream| {
        // Set both input and output rates to 44100.
        test_minimum_resampling_input_frames_set_stream_rates(stream, (44100_f64, 44100_f64));
        let frames: i64 = 100;
        assert_eq!(minimum_resampling_input_frames(&stream, frames), frames);
    });
}

fn test_minimum_resampling_input_frames_set_stream_rates(
    stream: &mut AudioUnitStream,
    rates: (f64, f64),
) {
    let (input_rate, output_rate) = rates;

    // Set stream output rate
    let mut raw = ffi::cubeb_stream_params::default();
    raw.format = ffi::CUBEB_SAMPLE_FLOAT32NE;
    raw.rate = output_rate as u32;
    raw.channels = 2;
    raw.layout = ffi::CUBEB_LAYOUT_STEREO;
    raw.prefs = ffi::CUBEB_STREAM_PREF_NONE;
    stream.output_stream_params = StreamParams::from(raw);

    // Set stream input rate
    stream.input_hw_rate = input_rate;
}

// output_callback
// ------------------------------------
// TODO

// set_device_info
// ------------------------------------
#[test]
fn test_set_device_info_to_unknown_input_device() {
    test_get_empty_stream(|stream| {
        // The input device info of the stream will be set to the system default if the predefined
        // input device is unknown.
        if let Ok(default_input) =
            test_set_device_info_and_get_default_device(stream, Scope::Input, kAudioObjectUnknown)
        {
            assert_eq!(stream.input_device.id, default_input);
            assert_eq!(
                stream.input_device.flags,
                device_flags::DEV_INPUT
                    | device_flags::DEV_SELECTED_DEFAULT
                    | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

#[test]
fn test_set_device_info_to_unknown_output_device() {
    test_get_empty_stream(|stream| {
        // The output device info of the stream will be set to the system default if the predefined
        // output device is unknown.
        if let Ok(default_output) =
            test_set_device_info_and_get_default_device(stream, Scope::Output, kAudioObjectUnknown)
        {
            assert_eq!(stream.output_device.id, default_output);
            assert_eq!(
                stream.output_device.flags,
                device_flags::DEV_OUTPUT
                    | device_flags::DEV_SELECTED_DEFAULT
                    | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

// FIXIT: Is it ok to set input device to kAudioObjectSystemObject ?
//        The flags will be DEV_INPUT if we do so,
//        but shouldn't it be DEV_INPUT | DEV_SYSTEM_DEFAULT at least ?
#[ignore]
#[test]
fn test_set_device_info_to_system_input_device() {
    test_get_empty_stream(|stream| {
        // Will the input device info of the stream be set to the system default if the predefined
        // input device is system device ?
        if let Ok(default_input) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Input,
            kAudioObjectSystemObject,
        ) {
            assert_eq!(
                stream.input_device.id,
                default_input /* or kAudioObjectSystemObject ? */
            );
            assert_eq!(
                stream.input_device.flags,
                device_flags::DEV_INPUT | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

// FIXIT: Is it ok to set output device to kAudioObjectSystemObject ?
//        The flags will be DEV_OUTPUT if we do so,
//        but shouldn't it be DEV_OUTPUT | DEV_SYSTEM_DEFAULT at least ?
#[ignore]
#[test]
fn test_set_device_info_to_system_output_device() {
    test_get_empty_stream(|stream| {
        // Will the output device info of the stream be set to the system default if the predefined
        // input device is system device ?
        if let Ok(default_output) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Output,
            kAudioObjectSystemObject,
        ) {
            assert_eq!(
                stream.output_device.id,
                default_output /* or kAudioObjectSystemObject ? */
            );
            assert_eq!(
                stream.output_device.flags,
                device_flags::DEV_OUTPUT | device_flags::DEV_SYSTEM_DEFAULT
            );
        }
    });
}

// FIXIT: Is it ok to set input device to a nonexistent device ?
#[ignore]
#[test]
fn test_set_device_info_to_nonexistent_input_device() {
    test_get_empty_stream(|stream| {
        if let Ok(_default_input) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Input,
            std::u32::MAX, // TODO: Create an API to get nonexistent device.
        ) {
            assert!(false);
        }
    });
}

// FIXIT: Is it ok to set output device to a nonexistent device ?
#[ignore]
#[test]
fn test_set_device_info_to_nonexistent_output_device() {
    test_get_empty_stream(|stream| {
        if let Ok(_default_output) = test_set_device_info_and_get_default_device(
            stream,
            Scope::Output,
            std::u32::MAX, // TODO: Create an API to get nonexistent device.
        ) {
            assert!(false);
        }
    });
}

fn test_set_device_info_and_get_default_device(
    stream: &mut AudioUnitStream,
    scope: Scope,
    predefined_device: AudioDeviceID,
) -> std::result::Result<AudioDeviceID, ()> {
    assert_eq!(stream.input_device.id, kAudioObjectUnknown);
    assert_eq!(stream.input_device.flags, device_flags::DEV_UNKNOWN);
    assert_eq!(stream.output_device.id, kAudioObjectUnknown);
    assert_eq!(stream.output_device.flags, device_flags::DEV_UNKNOWN);

    let default_device = test_get_default_device(scope.clone().into());
    // Fail to call audiounit_set_device_info when there is no available device.
    if default_device.is_none() {
        assert_eq!(
            audiounit_set_device_info(stream, predefined_device, scope.into()).unwrap_err(),
            Error::error()
        );
        return Err(());
    }

    // Set the device info to the predefined device
    assert!(audiounit_set_device_info(stream, predefined_device, scope.into()).is_ok());
    Ok(default_device.unwrap())
}

// reinit_stream
// ------------------------------------
// TODO

// reinit_stream_async
// ------------------------------------
// TODO

// event_addr_to_string
// ------------------------------------
// TODO

// property_listener_callback
// ------------------------------------
// TODO
