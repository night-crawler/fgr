#[macro_export]
macro_rules! mk_filter_enum {
    (
        $name:ident, $map:ident,
        [
            $(
                $opt_name:ident: $($alias:literal),+
            ),+
        ]
    ) => {

        #[derive(Debug, Eq, PartialEq, strum_macros::EnumIter, strum_macros::EnumString, strum_macros::IntoStaticStr)]
        pub enum $name {
            $(
                $opt_name,
            )+
        }

        lazy_static::lazy_static! {
            static ref $map: std::collections::BTreeMap<&'static str, &'static str> =
                $crate::parse::util::prepare_enum_map::<$name>();
        }

        impl $crate::parse::traits::AliasExt for $name {
            fn get_aliases(&self) -> (&'static [&'static str], &'static str) {
                match self {
                    $(
                        Self::$opt_name => (
                            &[
                                stringify!($opt_name),
                                $($alias),+
                            ],
                            stringify!($opt_name)
                        ),
                    )+
                }
            }

            fn split_by_longest_alias(input: &str) -> Option<(&str, &str)> {
                $crate::parse::util::split_by_longest_alias(input, $map.iter().rev())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$opt_name => write!(f, "{}", stringify!($opt_name)),
                    )+
                }
            }
        }

    }
}
