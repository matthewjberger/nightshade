#[macro_export]
macro_rules! config_kind {
    (toggle($default:expr)) => {
        $crate::contract::ConfigKind::Toggle { default: $default }
    };
    (number($minimum:expr, $maximum:expr, $default:expr)) => {
        $crate::contract::ConfigKind::Number {
            minimum: $minimum,
            maximum: $maximum,
            default: $default,
        }
    };
    (whole($minimum:expr, $maximum:expr, $default:expr)) => {
        $crate::contract::ConfigKind::Whole {
            minimum: $minimum,
            maximum: $maximum,
            default: $default,
        }
    };
    (choice($default:expr, $options:expr)) => {
        $crate::contract::ConfigKind::Choice {
            default: $default,
            options: &$options,
        }
    };
    (text($default:expr)) => {
        $crate::contract::ConfigKind::Text { default: $default }
    };
    (color($default:expr)) => {
        $crate::contract::ConfigKind::Color { default: $default }
    };
}

#[macro_export]
macro_rules! config {
    (
        $(#[$meta:meta])*
        $visibility:vis enum $name:ident {
            $(
                $section:literal {
                    $(
                        $variant:ident : $kind:ident ( $( $argument:expr ),* $(,)? )
                    ),* $(,)?
                }
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        $visibility enum $name {
            $( $( $variant, )* )*
        }

        impl $crate::contract::ConfigSchema for $name {
            fn entries() -> &'static [$crate::contract::ConfigEntry] {
                &[
                    $( $(
                        $crate::contract::ConfigEntry {
                            key: stringify!($variant),
                            section: $section,
                            kind: $crate::config_kind!($kind ( $( $argument ),* )),
                        },
                    )* )*
                ]
            }

            fn index(self) -> usize {
                self as usize
            }
        }
    };
}
