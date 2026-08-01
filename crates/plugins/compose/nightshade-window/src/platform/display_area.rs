#[cfg(target_os = "android")]
const FIRST_SDK_WITH_A_CUTOUT: i32 = 28;

#[cfg(target_os = "android")]
#[derive(Clone, Copy, Default)]
pub struct DisplayInsets {
    pub cutout: [i32; 4],
    pub bars: [i32; 4],
}

pub fn request_whole_display() {
    #[cfg(target_os = "android")]
    {
        use jni::objects::JObject;
        use jni::{JavaVM, jni_sig, jni_str};

        const CUTOUT_MODE: i32 = 3;

        if nightshade_platform::android::android_activity::AndroidApp::sdk_version()
            < FIRST_SDK_WITH_A_CUTOUT
        {
            return;
        }
        let Some(android_app) = nightshade_platform::android::android_app() else {
            return;
        };

        let vm = unsafe { JavaVM::from_raw(android_app.vm_as_ptr().cast()) };
        let asked: Result<(), jni::errors::Error> = vm.attach_current_thread(|env| {
            let activity = unsafe { JObject::from_raw(env, android_app.activity_as_ptr().cast()) };
            let window = env
                .call_method(
                    &activity,
                    jni_str!("getWindow"),
                    jni_sig!("()Landroid/view/Window;"),
                    &[],
                )?
                .l()?;
            let params = env
                .call_method(
                    &window,
                    jni_str!("getAttributes"),
                    jni_sig!("()Landroid/view/WindowManager$LayoutParams;"),
                    &[],
                )?
                .l()?;
            env.set_field(
                &params,
                jni_str!("layoutInDisplayCutoutMode"),
                jni_sig!("I"),
                CUTOUT_MODE.into(),
            )?;
            env.call_method(
                &window,
                jni_str!("setAttributes"),
                jni_sig!("(Landroid/view/WindowManager$LayoutParams;)V"),
                &[(&params).into()],
            )?;
            Ok(())
        });

        if let Err(error) = asked {
            tracing::warn!("the window would not take the whole display: {error}");
        }
    }
}

#[cfg(target_os = "android")]
pub fn display_insets() -> DisplayInsets {
    use jni::objects::JObject;
    use jni::{JavaVM, jni_sig, jni_str};

    if nightshade_platform::android::android_activity::AndroidApp::sdk_version()
        < FIRST_SDK_WITH_A_CUTOUT
    {
        return DisplayInsets::default();
    }
    let Some(android_app) = nightshade_platform::android::android_app() else {
        return DisplayInsets::default();
    };

    let vm = unsafe { JavaVM::from_raw(android_app.vm_as_ptr().cast()) };
    let read: Result<DisplayInsets, jni::errors::Error> = vm.attach_current_thread(|env| {
        let activity = unsafe { JObject::from_raw(env, android_app.activity_as_ptr().cast()) };
        let window = env
            .call_method(
                &activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        let decor = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let insets = env
            .call_method(
                &decor,
                jni_str!("getRootWindowInsets"),
                jni_sig!("()Landroid/view/WindowInsets;"),
                &[],
            )?
            .l()?;
        if insets.is_null() {
            return Ok(DisplayInsets::default());
        }

        let mut bars = [0i32; 4];
        for (slot, name) in bars.iter_mut().zip([
            jni_str!("getSystemWindowInsetLeft"),
            jni_str!("getSystemWindowInsetTop"),
            jni_str!("getSystemWindowInsetRight"),
            jni_str!("getSystemWindowInsetBottom"),
        ]) {
            *slot = env.call_method(&insets, name, jni_sig!("()I"), &[])?.i()?;
        }

        let cutout = env
            .call_method(
                &insets,
                jni_str!("getDisplayCutout"),
                jni_sig!("()Landroid/view/DisplayCutout;"),
                &[],
            )?
            .l()?;
        if cutout.is_null() {
            return Ok(DisplayInsets {
                cutout: [0; 4],
                bars,
            });
        }

        let mut hole = [0i32; 4];
        for (slot, name) in hole.iter_mut().zip([
            jni_str!("getSafeInsetLeft"),
            jni_str!("getSafeInsetTop"),
            jni_str!("getSafeInsetRight"),
            jni_str!("getSafeInsetBottom"),
        ]) {
            *slot = env.call_method(&cutout, name, jni_sig!("()I"), &[])?.i()?;
        }
        Ok(DisplayInsets { cutout: hole, bars })
    });

    match read {
        Ok(insets) => insets,
        Err(error) => {
            tracing::warn!("the display insets would not be read: {error}");
            DisplayInsets::default()
        }
    }
}
