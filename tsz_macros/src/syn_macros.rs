macro_rules! ast_struct {
    (
        [$($attrs_pub:tt)*]
        struct $name:ident #full $($rest:tt)*
    ) => {
        $($attrs_pub)* struct $name $($rest)*

        impl ::quote::ToTokens for $name {
            fn to_tokens(&self, _: &mut ::proc_macro2::TokenStream) {
                unreachable!()
            }
        }
    };

    (
        [$($attrs_pub:tt)*]
        struct $name:ident $($rest:tt)*
    ) => {
        $($attrs_pub)* struct $name $($rest)*
    };

    ($($t:tt)*) => {
        strip_attrs_pub!(ast_struct!($($t)*));
    };
}

pub(crate) use ast_struct; 

macro_rules! ast_enum {
    // Drop the `#no_visit` attribute, if present.
    (
        [$($attrs_pub:tt)*]
        enum $name:ident #no_visit $($rest:tt)*
    ) => (
        ast_enum!([$($attrs_pub)*] enum $name $($rest)*);
    );

    (
        [$($attrs_pub:tt)*]
        enum $name:ident $($rest:tt)*
    ) => (
        $($attrs_pub)* enum $name $($rest)*
    );

    ($($t:tt)*) => {
        strip_attrs_pub!(ast_enum!($($t)*));
    };
}

pub(crate) use ast_enum; 

macro_rules! ast_enum_of_structs {
    (
        $(#[$enum_attr:meta])*
        $pub:ident $enum:ident $name:ident $body:tt
        $($remaining:tt)*
    ) => {
        ast_enum!($(#[$enum_attr])* $pub $enum $name $body);
        ast_enum_of_structs_impl!($pub $enum $name $body $($remaining)*);
    };
}

pub(crate) use ast_enum_of_structs; 

macro_rules! ast_enum_of_structs_impl {
    (
        $pub:ident $enum:ident $name:ident {
            $(
                $(#[cfg $cfg_attr:tt])*
                $(#[doc $($doc_attr:tt)*])*
                $variant:ident $( ($($member:ident)::+) )*,
            )*
        }
    ) => {
        check_keyword_matches!(pub $pub);
        check_keyword_matches!(enum $enum);

        $($(
            ast_enum_from_struct!($name::$variant, $($member)::+);
        )*)*

        generate_to_tokens! {
            ()
            tokens
            $name {
                $(
                    $(#[cfg $cfg_attr])*
                    $(#[doc $($doc_attr)*])*
                    $variant $($($member)::+)*,
                )*
            }
        }
    };
}

pub(crate) use ast_enum_of_structs_impl; 

macro_rules! ast_enum_from_struct {
    // No From<TokenStream> for verbatim variants.
    ($name:ident::Verbatim, $member:ident) => {};

    ($name:ident::$variant:ident, $member:ident) => {
        impl From<$member> for $name {
            fn from(e: $member) -> $name {
                $name::$variant(e)
            }
        }
    };
}

pub(crate) use ast_enum_from_struct; 

macro_rules! generate_to_tokens {
    (
        ($($arms:tt)*) $tokens:ident $name:ident {
            $(#[cfg $cfg_attr:tt])*
            $(#[doc $($doc_attr:tt)*])*
            $variant:ident,
            $($next:tt)*
        }
    ) => {
        generate_to_tokens!(
            ($($arms)* $(#[cfg $cfg_attr])* $name::$variant => {})
            $tokens $name { $($next)* }
        );
    };

    (
        ($($arms:tt)*) $tokens:ident $name:ident {
            $(#[cfg $cfg_attr:tt])*
            $(#[doc $($doc_attr:tt)*])*
            $variant:ident $member:ident,
            $($next:tt)*
        }
    ) => {
        generate_to_tokens!(
            ($($arms)* $(#[cfg $cfg_attr])* $name::$variant(_e) => _e.to_tokens($tokens),)
            $tokens $name { $($next)* }
        );
    };

    (($($arms:tt)*) $tokens:ident $name:ident {}) => {
        #[cfg_attr(doc_cfg, doc(cfg(feature = "printing")))]
        impl ::quote::ToTokens for $name {
            fn to_tokens(&self, $tokens: &mut ::proc_macro2::TokenStream) {
                match self {
                    $($arms)*
                }
            }
        }
    };
}

pub(crate) use generate_to_tokens; 

macro_rules! strip_attrs_pub {
    ($mac:ident!($(#[$m:meta])* $pub:ident $($t:tt)*)) => {
        check_keyword_matches!(pub $pub);

        $mac!([$(#[$m])* $pub] $($t)*);
    };
}

pub(crate) use strip_attrs_pub; 

macro_rules! check_keyword_matches {
    (enum enum) => {};
    (pub pub) => {};
}

pub(crate) use check_keyword_matches; 