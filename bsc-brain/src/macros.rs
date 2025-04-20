#[macro_export]
macro_rules! main {
    ($main_type:ident) => {
        mod __brain_main {
            use bsc_brain::{FromApi, Main};

            static mut __BRAIN_MAIN: Option<(super::$main_type, bsc_brain::NativeApi)> = None;

            fn unsafe_brain_access()
            -> &'static mut Option<(super::$main_type, bsc_brain::NativeApi)> {
                unsafe { &mut (*&raw mut __BRAIN_MAIN) }
            }

            #[unsafe(no_mangle)]
            extern "C" fn brain_init() {
                let mut api = bsc_brain::NativeApi::new();
                let instance = <super::$main_type>::init(&mut api);
                *unsafe_brain_access() = Some((instance, api));
            }

            #[unsafe(no_mangle)]
            extern "C" fn brain_update() {
                if let Some((brain, api)) = unsafe_brain_access().as_mut() {
                    brain.update(api);
                }
            }

            #[unsafe(no_mangle)]
            extern "C" fn brain_shutdown() {
                *unsafe_brain_access() = None;
            }
        }
    };
}
