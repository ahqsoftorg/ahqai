#![feature(prelude_import)]
#![feature(duration_constructors)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::{env::args, panic, process};
mod log {
    use std::time::SystemTime;
    use chalk_rs::Chalk;
    use log::Level;
    pub fn setup() {
        let mut chalk = Chalk::new();
        let info = chalk.blue().string(&"INFO").leak();
        let warn = chalk.yellow().string(&"WARN").leak();
        let err = chalk.red().bold().string(&"ERRR").leak();
        let fe = fern::Dispatch::new()
            .format(|out, message, record| {
                let mut chalk = Chalk::new();
                let (level, msg) = match record.level() {
                    Level::Trace => ("TRCE", message.to_string()),
                    Level::Debug => ("DEBG", message.to_string()),
                    Level::Info => (&*info, chalk.blue().string(&message)),
                    Level::Warn => (&*warn, chalk.yellow().string(&message)),
                    Level::Error => (&*err, chalk.red().bold().string(&message)),
                };
                let target_str = record.target();
                let target = if target_str.starts_with("ahqai_server::") {
                    "".into()
                } else {
                    chalk
                        .reset_style()
                        .dim()
                        .string(
                            &::alloc::__export::must_use({
                                ::alloc::fmt::format(format_args!("({0})", target_str))
                            }),
                        )
                };
                out.finish(
                    format_args!(
                        "[{0} {1}] {2} {3}",
                        humantime::format_rfc3339_seconds(SystemTime::now()),
                        level,
                        msg,
                        target,
                    ),
                )
            });
        let fe = fe.level(log::LevelFilter::Debug);
        fe.chain(std::io::stdout()).apply().unwrap();
    }
}
mod server {
    use crate::{
        auth::{AuthSessionManager, argon::{self, server::verify_server_pass}},
        structs::{Authentication, Config},
    };
    use actix_web::{App, HttpServer, web};
    use chalk_rs::Chalk;
    use log::*;
    use secrecy::{ExposeSecret, SecretString};
    use serde_json::from_str;
    use std::{
        env, fs as stdfs, sync::{LazyLock, OnceLock},
        thread::{self, available_parallelism},
    };
    use tokio::sync::RwLock;
    use zeroize::Zeroize;
    pub mod admin {
        use actix_web::{
            HttpResponse, HttpResponseBuilder, Responder, Result, delete,
            http::StatusCode, post, web::Bytes,
        };
        use secrecy::ExposeSecret;
        use serde::Deserialize;
        use serde_json::from_slice;
        use tokio::task::yield_now;
        use async_stream::stream;
        use futures::Stream;
        use crate::{
            auth::{AccountCreateOutcome, AuthSessionManager},
            server::{AUTH, CONFIG, REAL_ADMIN_PASSWORD},
            structs::Authentication,
        };
        #[serde(deny_unknown_fields)]
        struct AdminAuthRequest<'a> {
            #[serde(borrow)]
            password: &'a str,
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de: 'a, 'a> _serde::Deserialize<'de> for AdminAuthRequest<'a> {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 1",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "password" => _serde::__private228::Ok(__Field::__field0),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"password" => _serde::__private228::Ok(__Field::__field0),
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de: 'a, 'a> {
                        marker: _serde::__private228::PhantomData<AdminAuthRequest<'a>>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de: 'a, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                        type Value = AdminAuthRequest<'a>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct AdminAuthRequest",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct AdminAuthRequest with 1 element",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(AdminAuthRequest {
                                password: __field0,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "password",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("password")?
                                }
                            };
                            _serde::__private228::Ok(AdminAuthRequest {
                                password: __field0,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["password"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "AdminAuthRequest",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<
                                AdminAuthRequest<'a>,
                            >,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        #[serde(deny_unknown_fields)]
        struct AdminSearchRequest<'a> {
            #[serde(borrow)]
            password: &'a str,
            search: String,
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de: 'a, 'a> _serde::Deserialize<'de> for AdminSearchRequest<'a> {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "password" => _serde::__private228::Ok(__Field::__field0),
                                "search" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"password" => _serde::__private228::Ok(__Field::__field0),
                                b"search" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de: 'a, 'a> {
                        marker: _serde::__private228::PhantomData<
                            AdminSearchRequest<'a>,
                        >,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de: 'a, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                        type Value = AdminSearchRequest<'a>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct AdminSearchRequest",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct AdminSearchRequest with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct AdminSearchRequest with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(AdminSearchRequest {
                                password: __field0,
                                search: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "password",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("search"),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("password")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("search")?
                                }
                            };
                            _serde::__private228::Ok(AdminSearchRequest {
                                password: __field0,
                                search: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["password", "search"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "AdminSearchRequest",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<
                                AdminSearchRequest<'a>,
                            >,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        #[serde(deny_unknown_fields)]
        struct AdminUserCreateRequest<'a> {
            #[serde(borrow)]
            password: &'a str,
            #[serde(borrow)]
            unique_id: &'a str,
            #[serde(borrow)]
            user_password: &'a str,
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de: 'a, 'a> _serde::Deserialize<'de> for AdminUserCreateRequest<'a> {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                2u64 => _serde::__private228::Ok(__Field::__field2),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 3",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "password" => _serde::__private228::Ok(__Field::__field0),
                                "unique_id" => _serde::__private228::Ok(__Field::__field1),
                                "user_password" => {
                                    _serde::__private228::Ok(__Field::__field2)
                                }
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"password" => _serde::__private228::Ok(__Field::__field0),
                                b"unique_id" => _serde::__private228::Ok(__Field::__field1),
                                b"user_password" => {
                                    _serde::__private228::Ok(__Field::__field2)
                                }
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de: 'a, 'a> {
                        marker: _serde::__private228::PhantomData<
                            AdminUserCreateRequest<'a>,
                        >,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de: 'a, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                        type Value = AdminUserCreateRequest<'a>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct AdminUserCreateRequest",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct AdminUserCreateRequest with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct AdminUserCreateRequest with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct AdminUserCreateRequest with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(AdminUserCreateRequest {
                                password: __field0,
                                unique_id: __field1,
                                user_password: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            let mut __field2: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "password",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "unique_id",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private228::Option::is_some(&__field2) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "user_password",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("password")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("unique_id")?
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private228::Some(__field2) => __field2,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("user_password")?
                                }
                            };
                            _serde::__private228::Ok(AdminUserCreateRequest {
                                password: __field0,
                                unique_id: __field1,
                                user_password: __field2,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "password",
                        "unique_id",
                        "user_password",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "AdminUserCreateRequest",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<
                                AdminUserCreateRequest<'a>,
                            >,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        #[serde(deny_unknown_fields)]
        struct AdminDeleteRequest<'a> {
            #[serde(borrow)]
            password: &'a str,
            unique_id: String,
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de: 'a, 'a> _serde::Deserialize<'de> for AdminDeleteRequest<'a> {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "password" => _serde::__private228::Ok(__Field::__field0),
                                "unique_id" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"password" => _serde::__private228::Ok(__Field::__field0),
                                b"unique_id" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de: 'a, 'a> {
                        marker: _serde::__private228::PhantomData<
                            AdminDeleteRequest<'a>,
                        >,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de: 'a, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                        type Value = AdminDeleteRequest<'a>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct AdminDeleteRequest",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct AdminDeleteRequest with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct AdminDeleteRequest with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(AdminDeleteRequest {
                                password: __field0,
                                unique_id: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "password",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "unique_id",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("password")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("unique_id")?
                                }
                            };
                            _serde::__private228::Ok(AdminDeleteRequest {
                                password: __field0,
                                unique_id: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["password", "unique_id"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "AdminDeleteRequest",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<
                                AdminDeleteRequest<'a>,
                            >,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        async fn verify_auth<'a>(passwd: &'a str) -> Result<(), HttpResponse> {
            let value = REAL_ADMIN_PASSWORD
                .get()
                .map(|x| async move { passwd == x.read().await.expose_secret() });
            let val;
            if let Some(v) = value {
                val = v.await;
            } else {
                val = false;
            }
            match val {
                true => Ok(()),
                _ => {
                    Err(
                        HttpResponse::Unauthorized().body(r#"{ "msg": "Unauthorized" }"#),
                    )
                }
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct verify;
        impl ::actix_web::dev::HttpServiceFactory for verify {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn verify(body: Bytes) -> Result<impl Responder> {
                    let auth: AdminAuthRequest = from_slice(&body)?;
                    if let Err(r) = verify_auth(auth.password).await {
                        return Ok(r);
                    }
                    Ok(HttpResponse::NoContent().body::<&[u8]>(&[]))
                }
                let __resource = ::actix_web::Resource::new("/admin/verify")
                    .name("verify")
                    .guard(::actix_web::guard::Post())
                    .to(verify);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct list;
        impl ::actix_web::dev::HttpServiceFactory for list {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn list(body: Bytes) -> Result<impl Responder> {
                    let data: AdminSearchRequest = from_slice(&body)?;
                    if let Err(r) = verify_auth(data.password).await {
                        return Ok(r);
                    }
                    if let Some(auth) = AUTH.get() {
                        return Ok(
                            HttpResponseBuilder::new(StatusCode::OK)
                                .streaming(user_list_stream(auth, data.search)),
                        );
                    }
                    Ok(
                        HttpResponse::ServiceUnavailable()
                            .body::<&[u8]>(br#"{ "msg": "Auth is disabled" }"#),
                    )
                }
                let __resource = ::actix_web::Resource::new("/admin/clients")
                    .name("list")
                    .guard(::actix_web::guard::Post())
                    .to(list);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
        fn user_list_stream<'a>(
            auth: &'static AuthSessionManager,
            prefix: String,
        ) -> impl Stream<Item = Result<Bytes>> {
            {
                let (mut __yield_tx, __yield_rx) = unsafe {
                    ::async_stream::__private::yielder::pair()
                };
                ::async_stream::__private::AsyncStream::new(
                    __yield_rx,
                    async move {
                        '__async_stream_private_check_scope: {
                            let mut index = 0usize;
                            for uid in match auth.accounts.search(prefix).await {
                                ::core::result::Result::Ok(v) => v,
                                ::core::result::Result::Err(e) => {
                                    __yield_tx
                                        .send(::core::result::Result::Err(e.into()))
                                        .await;
                                    return;
                                }
                            } {
                                if index != 0 {
                                    {
                                        #[allow(unreachable_code)]
                                        if false {
                                            break '__async_stream_private_check_scope (loop {});
                                        }
                                        __yield_tx.send(Ok(Bytes::from_static(b"\n"))).await
                                    };
                                }
                                {
                                    #[allow(unreachable_code)]
                                    if false {
                                        break '__async_stream_private_check_scope (loop {});
                                    }
                                    __yield_tx.send(Ok(Bytes::from_owner(uid))).await
                                };
                                if index % 30 == 0 {
                                    yield_now().await;
                                }
                                index += 1;
                            }
                        }
                    },
                )
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct create;
        impl ::actix_web::dev::HttpServiceFactory for create {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn create(body: Bytes) -> Result<impl Responder> {
                    let data: AdminUserCreateRequest = from_slice(&body)?;
                    if let Err(r) = verify_auth(data.password).await {
                        return Ok(r);
                    }
                    let Authentication::Account { .. } = CONFIG.authentication else {
                        return Ok(
                            HttpResponse::ServiceUnavailable()
                                .body::<&[u8]>(br#"{ "msg": "Auth is not account based" }"#),
                        );
                    };
                    if let Some(auth) = AUTH.get() {
                        return match auth
                            .register(data.unique_id, data.user_password)
                            .await?
                        {
                            AccountCreateOutcome::InternalServerError => {
                                Ok(
                                    HttpResponse::InternalServerError()
                                        .body(r#"{ "msg": "Internal Server Error" }"#),
                                )
                            }
                            AccountCreateOutcome::Successful => {
                                Ok(HttpResponse::NoContent().body(::alloc::vec::Vec::new()))
                            }
                            AccountCreateOutcome::UsernameExists => {
                                Ok(
                                    HttpResponse::Conflict()
                                        .body(r#"{ "msg": "User already exists" }"#),
                                )
                            }
                            AccountCreateOutcome::WeakPassword => {
                                Ok(
                                    HttpResponse::BadRequest()
                                        .body(r#"{ "msg": "Insecure Password" }"#),
                                )
                            }
                            _ => {
                                Ok(
                                    HttpResponse::UnprocessableEntity()
                                        .body(r#"{ "msg": "Unreachable Output" }"#),
                                )
                            }
                        };
                    }
                    Ok(
                        HttpResponse::ServiceUnavailable()
                            .body::<&[u8]>(br#"{ "msg": "Auth is disabled" }"#),
                    )
                }
                let __resource = ::actix_web::Resource::new("/admin/user")
                    .name("create")
                    .guard(::actix_web::guard::Post())
                    .to(create);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct create_token;
        impl ::actix_web::dev::HttpServiceFactory for create_token {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn create_token(body: Bytes) -> Result<impl Responder> {
                    let auth: AdminAuthRequest = from_slice(&body)?;
                    if let Err(r) = verify_auth(auth.password).await {
                        return Ok(r);
                    }
                    let Authentication::Account { .. } = CONFIG.authentication else {
                        return Ok(
                            HttpResponse::ServiceUnavailable()
                                .body::<&[u8]>(br#"{ "msg": "Auth is not account based" }"#),
                        );
                    };
                    if let Some(auth) = AUTH.get() {
                        return match auth.add_token().await? {
                            AccountCreateOutcome::InternalServerError => {
                                Ok(
                                    HttpResponse::InternalServerError()
                                        .body(r#"{ "msg": "Internal Server Error" }"#),
                                )
                            }
                            AccountCreateOutcome::SuccessfulOut(out) => {
                                Ok(HttpResponse::Ok().body(Bytes::from_owner(out)))
                            }
                            AccountCreateOutcome::UsernameExists => {
                                Ok(
                                    HttpResponse::Conflict()
                                        .body(r#"{ "msg": "User already exists" }"#),
                                )
                            }
                            AccountCreateOutcome::WeakPassword => {
                                Ok(
                                    HttpResponse::BadRequest()
                                        .body(r#"{ "msg": "Insecure Password" }"#),
                                )
                            }
                            _ => {
                                Ok(
                                    HttpResponse::UnprocessableEntity()
                                        .body(r#"{ "msg": "Unreachable Output" }"#),
                                )
                            }
                        };
                    }
                    Ok(
                        HttpResponse::ServiceUnavailable()
                            .body::<&[u8]>(br#"{ "msg": "Auth is disabled" }"#),
                    )
                }
                let __resource = ::actix_web::Resource::new("/admin/token")
                    .name("create_token")
                    .guard(::actix_web::guard::Post())
                    .to(create_token);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct delete;
        impl ::actix_web::dev::HttpServiceFactory for delete {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn delete(body: Bytes) -> Result<impl Responder> {
                    let data: AdminDeleteRequest = from_slice(&body)?;
                    if let Err(r) = verify_auth(data.password).await {
                        return Ok(r);
                    }
                    let Authentication::Account { .. } = CONFIG.authentication else {
                        return Ok(
                            HttpResponse::ServiceUnavailable()
                                .body::<&[u8]>(br#"{ "msg": "Auth is not account based" }"#),
                        );
                    };
                    if let Some(auth) = AUTH.get() {
                        _ = auth.accounts.remove(data.unique_id).await;
                        return Ok(HttpResponse::NoContent().body::<&[u8]>(&[]));
                    }
                    Ok(
                        HttpResponse::ServiceUnavailable()
                            .body::<&[u8]>(br#"{ "msg": "Auth is disabled" }"#),
                    )
                }
                let __resource = ::actix_web::Resource::new("/admin/client")
                    .name("delete")
                    .guard(::actix_web::guard::Delete())
                    .to(delete);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
    }
    pub mod auth {
        use actix_web::{HttpResponse, Responder, Result, post, web::Bytes};
        use serde::Deserialize;
        use crate::{
            auth::{AccountCheckOutcome, AccountCreateOutcome},
            server::AUTH,
        };
        #[serde(deny_unknown_fields)]
        struct Auth<'a> {
            #[serde(borrow)]
            username: Option<&'a str>,
            #[serde(borrow)]
            pass: &'a str,
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de: 'a, 'a> _serde::Deserialize<'de> for Auth<'a> {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "username" => _serde::__private228::Ok(__Field::__field0),
                                "pass" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"username" => _serde::__private228::Ok(__Field::__field0),
                                b"pass" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de: 'a, 'a> {
                        marker: _serde::__private228::PhantomData<Auth<'a>>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de: 'a, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                        type Value = Auth<'a>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct Auth",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                Option<&'a str>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Auth with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct Auth with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(Auth {
                                username: __field0,
                                pass: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<
                                Option<&'a str>,
                            > = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "username",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<&'a str>,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("pass"),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("username")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("pass")?
                                }
                            };
                            _serde::__private228::Ok(Auth {
                                username: __field0,
                                pass: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["username", "pass"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Auth",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<Auth<'a>>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        #[serde(deny_unknown_fields)]
        struct AuthRegn<'a> {
            #[serde(borrow)]
            username: &'a str,
            #[serde(borrow)]
            pass: &'a str,
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de: 'a, 'a> _serde::Deserialize<'de> for AuthRegn<'a> {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"field index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "username" => _serde::__private228::Ok(__Field::__field0),
                                "pass" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"username" => _serde::__private228::Ok(__Field::__field0),
                                b"pass" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_field(__value, FIELDS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de: 'a, 'a> {
                        marker: _serde::__private228::PhantomData<AuthRegn<'a>>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de: 'a, 'a> _serde::de::Visitor<'de> for __Visitor<'de, 'a> {
                        type Value = AuthRegn<'a>;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct AuthRegn",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct AuthRegn with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                &'a str,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct AuthRegn with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(AuthRegn {
                                username: __field0,
                                pass: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<&'a str> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "username",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("pass"),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<&'a str>(&mut __map)?,
                                        );
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("username")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("pass")?
                                }
                            };
                            _serde::__private228::Ok(AuthRegn {
                                username: __field0,
                                pass: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["username", "pass"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "AuthRegn",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<AuthRegn<'a>>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        #[allow(non_camel_case_types, missing_docs)]
        pub struct auth;
        impl ::actix_web::dev::HttpServiceFactory for auth {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                pub async fn auth(payload: Bytes) -> Result<impl Responder> {
                    let Ok(auth) = serde_json::from_slice::<Auth>(&payload) else {
                        return Ok(
                            HttpResponse::BadRequest()
                                .body(r#"{ "msg": "Invalid Data" }"#),
                        );
                    };
                    let auth_ref = AUTH
                        .get()
                        .expect(
                            "Auth must be defined or else this function cant be registered",
                        );
                    let resp = match auth.username {
                        None => auth_ref.is_valid_token(auth.pass).await?,
                        Some(username) => {
                            auth_ref.is_valid_account(username, auth.pass).await?
                        }
                    };
                    match resp {
                        AccountCheckOutcome::Some(x) => Ok(HttpResponse::Ok().body(x)),
                        AccountCheckOutcome::InvalidPassword
                        | AccountCheckOutcome::NotFound => {
                            Ok(
                                HttpResponse::Unauthorized()
                                    .body("{\"msg\": \"Invalid Credentials\"}"),
                            )
                        }
                        AccountCheckOutcome::TooManyRequests => {
                            Ok(
                                HttpResponse::TooManyRequests()
                                    .body("{\"msg\": \"Too Many Requests\"}"),
                            )
                        }
                    }
                }
                let __resource = ::actix_web::Resource::new("/login")
                    .name("auth")
                    .guard(::actix_web::guard::Post())
                    .to(auth);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct register;
        impl ::actix_web::dev::HttpServiceFactory for register {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                pub async fn register(payload: Bytes) -> Result<impl Responder> {
                    let Ok(regn) = serde_json::from_slice::<AuthRegn>(&payload) else {
                        return Ok(
                            HttpResponse::BadRequest()
                                .body(r#"{ "msg": "Invalid Data" }"#),
                        );
                    };
                    let auth_ref = AUTH
                        .get()
                        .expect(
                            "Auth must be defined or else this function cant be registered",
                        );
                    if !auth_ref.can_register().await {
                        return Ok(
                            HttpResponse::UnprocessableEntity()
                                .body(
                                    r#"{ "msg": "Registration is disabled due to maximum user saturation" }"#,
                                ),
                        );
                    }
                    match auth_ref.register(regn.username, regn.pass).await? {
                        AccountCreateOutcome::InternalServerError => {
                            Ok(
                                HttpResponse::InternalServerError()
                                    .body(r#"{ "msg": "Internal Server Error" }"#),
                            )
                        }
                        AccountCreateOutcome::Successful => {
                            Ok(HttpResponse::NoContent().body(::alloc::vec::Vec::new()))
                        }
                        AccountCreateOutcome::UsernameExists => {
                            Ok(
                                HttpResponse::Conflict()
                                    .body(r#"{ "msg": "User already exists" }"#),
                            )
                        }
                        AccountCreateOutcome::WeakPassword => {
                            Ok(
                                HttpResponse::BadRequest()
                                    .body(r#"{ "msg": "Insecure Password" }"#),
                            )
                        }
                        _ => {
                            Ok(
                                HttpResponse::UnprocessableEntity()
                                    .body(r#"{ "msg": "Unreachable Output" }"#),
                            )
                        }
                    }
                }
                let __resource = ::actix_web::Resource::new("/register")
                    .name("register")
                    .guard(::actix_web::guard::Post())
                    .to(register);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
    }
    pub mod http {
        use actix_web::{
            HttpResponse, Responder, Result, get, http::header::ContentType, post,
            web::Bytes,
        };
        use crate::{auth::AGENT, server::{AUTH, http::structs::ROOT_RESPONSE_DATA}};
        pub mod structs {
            use std::{collections::HashMap, sync::LazyLock};
            use serde::Serialize;
            use crate::{server::CONFIG, structs::Authentication};
            pub enum ShowedAuth {
                OpenToAll,
                Account,
            }
            #[doc(hidden)]
            #[allow(
                non_upper_case_globals,
                unused_attributes,
                unused_qualifications,
                clippy::absolute_paths,
            )]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for ShowedAuth {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                    {
                        match *self {
                            ShowedAuth::OpenToAll => {
                                _serde::Serializer::serialize_unit_variant(
                                    __serializer,
                                    "ShowedAuth",
                                    0u32,
                                    "OpenToAll",
                                )
                            }
                            ShowedAuth::Account => {
                                _serde::Serializer::serialize_unit_variant(
                                    __serializer,
                                    "ShowedAuth",
                                    1u32,
                                    "Account",
                                )
                            }
                        }
                    }
                }
            };
            pub static ROOT_RESPONSE_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| {
                let root_response = RootResponse::new();
                serde_json::to_vec(&root_response)
                    .expect("Failed to serialize static RootResponse")
            });
            pub struct RootResponse {
                version: &'static str,
                auth: ShowedAuth,
                can_register: bool,
                models: HashMap<Box<str>, u16>,
            }
            #[doc(hidden)]
            #[allow(
                non_upper_case_globals,
                unused_attributes,
                unused_qualifications,
                clippy::absolute_paths,
            )]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for RootResponse {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                    {
                        let mut __serde_state = _serde::Serializer::serialize_struct(
                            __serializer,
                            "RootResponse",
                            false as usize + 1 + 1 + 1 + 1,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "version",
                            &self.version,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "auth",
                            &self.auth,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "can_register",
                            &self.can_register,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "models",
                            &self.models,
                        )?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
            impl RootResponse {
                pub fn new() -> Self {
                    let mut out = Self {
                        version: "0.3.2",
                        auth: ShowedAuth::OpenToAll,
                        can_register: false,
                        models: HashMap::new(),
                    };
                    match CONFIG.authentication {
                        Authentication::Account { registration_allowed, .. } => {
                            out.can_register = registration_allowed;
                            out.auth = ShowedAuth::Account;
                        }
                        Authentication::OpenToAll => {
                            out.auth = ShowedAuth::OpenToAll;
                        }
                    }
                    CONFIG
                        .llama
                        .models
                        .iter()
                        .for_each(|(key, value)| {
                            _ = out.models.insert(key.to_owned(), value.capabilities.0);
                        });
                    out
                }
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct index;
        impl ::actix_web::dev::HttpServiceFactory for index {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn index() -> impl Responder {
                    HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body::<&[u8]>(ROOT_RESPONSE_DATA.as_ref())
                }
                let __resource = ::actix_web::Resource::new("/")
                    .name("index")
                    .guard(::actix_web::guard::Get())
                    .to(index);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct me;
        impl ::actix_web::dev::HttpServiceFactory for me {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn me(payload: Bytes) -> Result<impl Responder> {
                    let session = str::from_utf8(&payload);
                    match session {
                        Ok(session) => {
                            let auth_ref = AUTH
                                .get()
                                .expect(
                                    "Auth must be defined or else this function cant be registered",
                                );
                            if auth_ref.verify_session(session).await {
                                Ok(HttpResponse::Ok().body::<&[u8]>(br#"{ "msg": "Ok" }"#))
                            } else {
                                Ok(
                                    HttpResponse::Unauthorized()
                                        .body::<&[u8]>(br#"{ "msg": "Unauthorized" }"#),
                                )
                            }
                        }
                        _ => {
                            Ok(
                                HttpResponse::BadRequest()
                                    .body::<&[u8]>(br#"{ "msg": "Bad Request" }"#),
                            )
                        }
                    }
                }
                let __resource = ::actix_web::Resource::new("/me")
                    .name("me")
                    .guard(::actix_web::guard::Get())
                    .to(me);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
        #[allow(non_camel_case_types, missing_docs)]
        pub struct challenge;
        impl ::actix_web::dev::HttpServiceFactory for challenge {
            fn register(self, __config: &mut actix_web::dev::AppService) {
                async fn challenge(payload: Bytes) -> Result<impl Responder> {
                    match AGENT.gen_signature(&payload).await {
                        Some(x) => Ok(HttpResponse::Ok().body(x.to_vec())),
                        _ => {
                            Ok(
                                HttpResponse::InternalServerError()
                                    .body::<&[u8]>(br#"{ "msg": "Unable to hash" }"#),
                            )
                        }
                    }
                }
                let __resource = ::actix_web::Resource::new("/challenge")
                    .name("challenge")
                    .guard(::actix_web::guard::Post())
                    .to(challenge);
                ::actix_web::dev::HttpServiceFactory::register(__resource, __config);
            }
        }
    }
    pub mod llama {}
    pub mod ffi {}
    pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
        let data = stdfs::read_to_string("config.json").expect("Unable to load config");
        from_str(&data).expect("Invalid configuration file, unable to parse")
    });
    pub static DECRYPTED_CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| decrypt_config());
    pub static AUTH: OnceLock<AuthSessionManager> = OnceLock::new();
    pub static REAL_ADMIN_PASSWORD: OnceLock<RwLock<SecretString>> = OnceLock::new();
    fn decrypt_config() -> RwLock<Config> {
        if let Some(pass) = REAL_ADMIN_PASSWORD.get() {
            let mut conf = CONFIG.clone();
            let conf = thread::spawn(move || {
                    argon::decrypt_config(
                        pass.blocking_read().expose_secret(),
                        &mut conf,
                    );
                    conf
                })
                .join()
                .expect("Unable to decrypt config");
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!("Successfully decrypted configuration"),
                            lvl,
                            &(
                                "ahqai_server::server",
                                "ahqai_server::server",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            return RwLock::new(conf);
        }
        {
            {
                let lvl = ::log::Level::Warn;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        { ::log::__private_api::GlobalLogger },
                        format_args!(
                            "No Server Administrator Password found to perform decryption, double check if this is a bug. If it is, create an issue immediately at https://github.com/ahqsoftorg/ahqai/issues",
                        ),
                        lvl,
                        &(
                            "ahqai_server::server",
                            "ahqai_server::server",
                            ::log::__private_api::loc(),
                        ),
                        (),
                    );
                }
            }
        };
        RwLock::new(CONFIG.clone())
    }
    pub fn launch() -> Chalk {
        let mut chalk = Chalk::new();
        {
            {
                let lvl = ::log::Level::Info;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        { ::log::__private_api::GlobalLogger },
                        format_args!("AHQ-AI Server v{0}", "0.3.2"),
                        lvl,
                        &(
                            "ahqai_server::server",
                            "ahqai_server::server",
                            ::log::__private_api::loc(),
                        ),
                        (),
                    );
                }
            }
        };
        chalk.reset_style();
        chalk
    }
    pub fn main() -> std::io::Result<()> {
        <::actix_web::rt::System>::new()
            .block_on(async move {
                {
                    let mut chalk = launch();
                    let admin_api = request_admin_passwd();
                    {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!("Decrypting configuration..."),
                                    lvl,
                                    &(
                                        "ahqai_server::server",
                                        "ahqai_server::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    _ = DECRYPTED_CONFIG.read().await;
                    {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!("Decryption successful..."),
                                    lvl,
                                    &(
                                        "ahqai_server::server",
                                        "ahqai_server::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    let auth = !#[allow(non_exhaustive_omitted_patterns)]
                    match CONFIG.authentication {
                        Authentication::OpenToAll => true,
                        _ => false,
                    };
                    let mut registration_api = false;
                    if auth {
                        {
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!(
                                            "Starting up authentication manager using the decrypted configuration.",
                                        ),
                                        lvl,
                                        &(
                                            "ahqai_server::server",
                                            "ahqai_server::server",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                        if let Authentication::Account { registration_allowed, .. } = &CONFIG
                            .authentication
                        {
                            registration_api = *registration_allowed;
                        }
                        _ = AUTH.set(AuthSessionManager::create().await);
                    }
                    let mut server = HttpServer::new(move || {
                            let mut app = App::new()
                                .service(http::index)
                                .service(http::challenge)
                                .service(http::me);
                            let auth = !#[allow(non_exhaustive_omitted_patterns)]
                            match CONFIG.authentication {
                                Authentication::OpenToAll => true,
                                _ => false,
                            };
                            if auth {
                                app = app.service(auth::auth);
                            }
                            if admin_api {
                                app = app
                                    .service(admin::verify)
                                    .service(admin::list)
                                    .service(admin::create)
                                    .service(admin::create_token)
                                    .service(admin::delete);
                            }
                            if registration_api {
                                app = app.service(auth::register);
                            }
                            app
                        })
                        .workers(available_parallelism()?.get());
                    for (host, port) in &CONFIG.binds {
                        {
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!("Binding to {0}:{1}", host, port),
                                        lvl,
                                        &(
                                            "ahqai_server::server",
                                            "ahqai_server::server",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                        server = server.bind((host as &str, *port))?;
                    }
                    {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!("Server is starting"),
                                    lvl,
                                    &(
                                        "ahqai_server::server",
                                        "ahqai_server::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    let out = server.run().await;
                    if let Err(e) = &out {
                        {
                            {
                                let lvl = ::log::Level::Error;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!("Server exited in an error state."),
                                        lvl,
                                        &(
                                            "ahqai_server::server",
                                            "ahqai_server::server",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                        {
                            {
                                let lvl = ::log::Level::Error;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!("{0}", e),
                                        lvl,
                                        &(
                                            "ahqai_server::server",
                                            "ahqai_server::server",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                    }
                    {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!(
                                        "Zeroizing the decrypted configuration and server administrator key",
                                    ),
                                    lvl,
                                    &(
                                        "ahqai_server::server",
                                        "ahqai_server::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    DECRYPTED_CONFIG.write().await.zeroize();
                    if let Some(x) = REAL_ADMIN_PASSWORD.get() {
                        x.write().await.zeroize();
                    }
                    {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!("Zeroized successfully"),
                                    lvl,
                                    &(
                                        "ahqai_server::server",
                                        "ahqai_server::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    {
                        {
                            let lvl = ::log::Level::Warn;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!(
                                        "Ctrl+C detected (most probably). Starting shutdown procedure. This might take a while.",
                                    ),
                                    lvl,
                                    &(
                                        "ahqai_server::server",
                                        "ahqai_server::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!(
                                        "{0}",
                                        chalk
                                            .red()
                                            .bold()
                                            .string(
                                                &"Please DO NOT use Ctrl+C to terminate. It will lead to data corruption!",
                                            ),
                                    ),
                                    lvl,
                                    &(
                                        "ahqai_server::server",
                                        "ahqai_server::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    out
                }
            })
    }
    fn request_admin_passwd() -> bool {
        if let Some(x) = &CONFIG.admin_pass_hash {
            let hash = x as &str;
            let passwd;
            if let Ok(x) = env::var("AHQAI_ADMIN_PASSWORD") {
                passwd = x;
            } else {
                {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                { ::log::__private_api::GlobalLogger },
                                format_args!("----------------"),
                                lvl,
                                &(
                                    "ahqai_server::server",
                                    "ahqai_server::server",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    }
                };
                {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                { ::log::__private_api::GlobalLogger },
                                format_args!(
                                    "THE GIVEN SERVER IS PROTECTED BY SERVER ADMIN PASSWORD",
                                ),
                                lvl,
                                &(
                                    "ahqai_server::server",
                                    "ahqai_server::server",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    }
                };
                {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                { ::log::__private_api::GlobalLogger },
                                format_args!(
                                    "BUT THE `AHQAI_ADMIN_PASSWORD` VARIABLE WAS NOT FOUND",
                                ),
                                lvl,
                                &(
                                    "ahqai_server::server",
                                    "ahqai_server::server",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    }
                };
                {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                { ::log::__private_api::GlobalLogger },
                                format_args!(
                                    "IN THE CURRENT SERVER ENVIRONMENT. REQUESTING MANUAL ENTRY",
                                ),
                                lvl,
                                &(
                                    "ahqai_server::server",
                                    "ahqai_server::server",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    }
                };
                {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                { ::log::__private_api::GlobalLogger },
                                format_args!("----------------"),
                                lvl,
                                &(
                                    "ahqai_server::server",
                                    "ahqai_server::server",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    }
                };
                {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api::log(
                                { ::log::__private_api::GlobalLogger },
                                format_args!(""),
                                lvl,
                                &(
                                    "ahqai_server::server",
                                    "ahqai_server::server",
                                    ::log::__private_api::loc(),
                                ),
                                (),
                            );
                        }
                    }
                };
                passwd = rpassword::prompt_password(
                        "Enter your administrator password : ",
                    )
                    .expect("Unable to read your password");
            }
            if !verify_server_pass(&passwd, hash).unwrap_or(false) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Invalid Password was provided"),
                    );
                }
            }
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!(""),
                            lvl,
                            &(
                                "ahqai_server::server",
                                "ahqai_server::server",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!("----------------"),
                            lvl,
                            &(
                                "ahqai_server::server",
                                "ahqai_server::server",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!("SERVER ADMIN PASSWORD AUTH SUCCESSFUL"),
                            lvl,
                            &(
                                "ahqai_server::server",
                                "ahqai_server::server",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!("SERVER WILL START UP NOW"),
                            lvl,
                            &(
                                "ahqai_server::server",
                                "ahqai_server::server",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!("----------------"),
                            lvl,
                            &(
                                "ahqai_server::server",
                                "ahqai_server::server",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            { ::log::__private_api::GlobalLogger },
                            format_args!(""),
                            lvl,
                            &(
                                "ahqai_server::server",
                                "ahqai_server::server",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                }
            };
            REAL_ADMIN_PASSWORD
                .set(RwLock::new(SecretString::from(passwd)))
                .expect("Impossible Error");
            return true;
        }
        false
    }
}
mod ui {
    use std::{
        env::home_dir, fs, ops::{Deref, DerefMut},
        sync::LazyLock, time::{SystemTime, UNIX_EPOCH},
    };
    use cursive::{
        Cursive, CursiveExt, align::Align, theme::{Effect, PaletteColor, Style, Theme},
        view::{Nameable, Resizable},
        views::{
            Button, Dialog, DummyView, EditView, LinearLayout, ScrollView, SelectView,
            TextView,
        },
    };
    use cursive_tabs::TabPanel;
    use serde_json::to_string_pretty;
    use tokio::runtime::{Builder, Runtime};
    use crate::{
        auth::argon::{migrate_config, server::{hash_server_pass, verify_server_pass}},
        structs::{Authentication, Config},
    };
    mod auth {
        use cursive::{
            Cursive, view::Nameable, views::{LinearLayout, NamedView, ScrollView},
        };
        use crate::{
            structs::{Authentication, Config},
            ui::{Ptr, lazy::OnAuthStateUpdate},
        };
        mod open {
            use cursive::{
                align::Align, theme::{Effect, Style},
                view::Resizable, views::{Button, DummyView, LinearLayout, TextView},
            };
            pub fn render(l: &mut LinearLayout) {
                l.add_child(
                    LinearLayout::horizontal()
                        .child(TextView::new("⚒ Authentication Type").full_width())
                        .child(Button::new_raw("No Auth (OpenToAll)", |_| {})),
                );
                l.add_child(DummyView::new().fixed_height(2));
                l.add_child(
                    TextView::new("No Auth")
                        .align(Align::center())
                        .style(
                            Style::merge(&[Effect::Dim.into(), Effect::Underline.into()]),
                        ),
                );
                l.add_child(
                    TextView::new(
                        "This means that the application requires ABSOLUTELY no authentication to talk to the api. This is only recommended for completely OFFLINE (DISCONNECTED FROM INTERNET) servers and must not be used for remote servers",
                    ),
                );
            }
        }
        mod user {
            use cursive::{
                align::Align, theme::{Effect, Style},
                view::{Nameable, Resizable},
                views::{
                    Button, Dialog, DummyView, EditView, LinearLayout, SelectView,
                    TextView,
                },
            };
            use crate::{
                structs::{Authentication, Config},
                ui::Ptr,
            };
            pub fn render(
                l: &mut LinearLayout,
                can_register: bool,
                memory: u32,
                time: u32,
            ) {
                l.add_child(
                    LinearLayout::horizontal()
                        .child(TextView::new("⚒ Authentication Type").full_width())
                        .child(Button::new_raw("Account Authentication", |_| {})),
                );
                l.add_child(
                    LinearLayout::horizontal()
                        .child(
                            TextView::new("⚒ Self Registration Allowed").full_width(),
                        )
                        .child(
                            Button::new_raw(
                                    if can_register { "[Yes]" } else { "[No]" },
                                    |x| {
                                        x.add_layer(
                                            Dialog::around(
                                                    SelectView::new()
                                                        .item("Yes", true)
                                                        .item("No", false)
                                                        .on_submit(|x, val| {
                                                            let state: &mut Ptr<Config> = x.user_data().unwrap();
                                                            if let Authentication::Account {
                                                                registration_allowed,
                                                                ..
                                                            } = &mut state.authentication
                                                            {
                                                                *registration_allowed = *val;
                                                            }
                                                            let val_f = *val;
                                                            x.call_on_name(
                                                                "user_reg_allowed",
                                                                move |x: &mut Button| {
                                                                    x.set_label_raw(if val_f { "[Yes]" } else { "[No]" });
                                                                },
                                                            );
                                                            x.pop_layer();
                                                        }),
                                                )
                                                .title("Self Registration"),
                                        );
                                    },
                                )
                                .with_name("user_reg_allowed"),
                        ),
                );
                l.add_child(DummyView::new().fixed_height(2));
                l.add_child(
                    TextView::new("⚒ Argon2")
                        .style(Style::merge(&[Effect::Underline.into()])),
                );
                l.add_child(
                    LinearLayout::horizontal()
                        .child(TextView::new("⚒ Argon2 Memory Cost").full_width())
                        .child(
                            Button::new_raw(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(format_args!("[{0} MiB]", memory))
                                    }),
                                    |x| {
                                        x.add_layer(
                                            Dialog::around(
                                                    EditView::new()
                                                        .on_edit(|x, val, _| {
                                                            let state: &mut Ptr<Config> = x.user_data().unwrap();
                                                            if let Ok(num) = val.parse::<u32>() {
                                                                if num > 0 {
                                                                    let Authentication::Account { max_memory, .. } = &mut state
                                                                        .authentication else {
                                                                        ::core::panicking::panic(
                                                                            "internal error: entered unreachable code",
                                                                        )
                                                                    };
                                                                    *max_memory = num;
                                                                    x.call_on_name(
                                                                        "ram_usage",
                                                                        move |x: &mut Button| {
                                                                            x.set_label_raw(
                                                                                ::alloc::__export::must_use({
                                                                                    ::alloc::fmt::format(format_args!("[{0} MiB]", num))
                                                                                }),
                                                                            );
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                        })
                                                        .on_submit(|x, _| {
                                                            x.pop_layer();
                                                        }),
                                                )
                                                .dismiss_button("Done")
                                                .title("Memory Cost"),
                                        );
                                    },
                                )
                                .with_name("ram_usage"),
                        ),
                );
                l.add_child(
                    LinearLayout::horizontal()
                        .child(
                            TextView::new("⚒ Argon2 Time Cost (Total Rounds)")
                                .full_width(),
                        )
                        .child(
                            Button::new_raw(
                                    ::alloc::__export::must_use({
                                        ::alloc::fmt::format(format_args!("<{0}>", time))
                                    }),
                                    |x| {
                                        x.add_layer(
                                            Dialog::around(
                                                    EditView::new()
                                                        .on_edit(|x, val, _| {
                                                            let state: &mut Ptr<Config> = x.user_data().unwrap();
                                                            if let Ok(num) = val.parse::<u32>() {
                                                                if num > 0 {
                                                                    let Authentication::Account { time_cost, .. } = &mut state
                                                                        .authentication else {
                                                                        ::core::panicking::panic(
                                                                            "internal error: entered unreachable code",
                                                                        )
                                                                    };
                                                                    *time_cost = num;
                                                                    x.call_on_name(
                                                                        "time",
                                                                        move |x: &mut Button| {
                                                                            x.set_label_raw(
                                                                                ::alloc::__export::must_use({
                                                                                    ::alloc::fmt::format(format_args!("<{0}>", num))
                                                                                }),
                                                                            );
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                        })
                                                        .on_submit(|x, _| {
                                                            x.pop_layer();
                                                        }),
                                                )
                                                .dismiss_button("Done")
                                                .title("Time Cost"),
                                        );
                                    },
                                )
                                .with_name("time"),
                        ),
                );
                l.add_child(DummyView::new().fixed_height(2));
                l.add_child(
                    TextView::new("Miscellaneous")
                        .style(Style::merge(&[Effect::Underline.into()])),
                );
                l.add_child(
                    LinearLayout::horizontal()
                        .child(TextView::new("⚒ Account Manager").full_width())
                        .child(
                            Button::new_raw(
                                "Use the admin binary ↗",
                                |x| {
                                    x.add_layer(
                                        Dialog::around(
                                                TextView::new(
                                                    "AHQ AI team provides a dedicated cli application to manage server users (accounts and tokens) for the whole AHQ AI server. You should look into that. Also, you may review the source code to obtain the api endpoints to manage these.",
                                                ),
                                            )
                                            .title("Server Administrator Portal")
                                            .dismiss_button("Ok")
                                            .min_width(32)
                                            .max_width(64),
                                    );
                                },
                            ),
                        ),
                );
                l.add_child(DummyView::new().fixed_height(2));
                l.add_child(
                    TextView::new("About User Auth")
                        .align(Align::center())
                        .style(
                            Style::merge(&[Effect::Dim.into(), Effect::Underline.into()]),
                        ),
                );
                l.add_child(
                    TextView::new(
                        "The Client application will be needed to provide a userid and password. This is the recommended authentication type for internet or LAN servers.",
                    ),
                );
            }
        }
        #[allow(clippy::type_complexity)]
        pub fn auth_page(
            siv: &mut Cursive,
        ) -> NamedView<
            OnAuthStateUpdate<
                NamedView<ScrollView<NamedView<LinearLayout>>>,
                impl Fn(&mut Cursive) + 'static,
            >,
        > {
            let layout = LinearLayout::vertical().with_name("authpage");
            OnAuthStateUpdate::new(
                    ScrollView::new(layout)
                        .show_scrollbars(true)
                        .with_name("⚒ Authentication"),
                    siv,
                    |x: &mut Cursive| {
                        let state: &mut Ptr<Config> = x.user_data().unwrap();
                        let auth = state.authentication.clone();
                        _ = x
                            .call_on_name(
                                "authpage",
                                |layout: &mut LinearLayout| {
                                    layout.clear();
                                    match auth {
                                        Authentication::OpenToAll => open::render(layout),
                                        Authentication::Account {
                                            registration_allowed,
                                            max_memory,
                                            time_cost,
                                        } => {
                                            user::render(
                                                layout,
                                                registration_allowed,
                                                max_memory,
                                                time_cost,
                                            )
                                        }
                                    }
                                },
                            );
                    },
                )
                .with_name("⚒ Authentication")
        }
    }
    mod bind {
        use cursive::{
            align::Align, theme::{Effect, Style},
            view::{Nameable, Resizable},
            views::{
                Button, Dialog, EditView, LinearLayout, NamedView, ResizedView,
                ScrollView, TextView,
            },
        };
        use crate::{structs::Config, ui::Ptr};
        pub fn bind(s: Ptr<Config>) -> ResizedView<Dialog> {
            Dialog::new()
                .title("Hosts and Ports")
                .content(ScrollView::new(gen_cnt(s.clone())).show_scrollbars(true))
                .button(
                    "Add",
                    |x| {
                        x.add_layer(add_binding());
                    },
                )
                .dismiss_button("Done")
                .full_screen()
        }
        fn add_binding() -> Dialog {
            Dialog::new()
                .content(
                    ScrollView::new(
                            LinearLayout::vertical()
                                .child(TextView::new("Enter hostname"))
                                .child(EditView::new().with_name("host"))
                                .child(TextView::new("Enter port"))
                                .child(
                                    EditView::new().max_content_width(5).with_name("port"),
                                ),
                        )
                        .show_scrollbars(true),
                )
                .button(
                    "Add",
                    |x| {
                        let host = x
                            .call_on_name("host", |x: &mut EditView| x.get_content())
                            .unwrap();
                        let port = x
                            .call_on_name("port", |x: &mut EditView| x.get_content())
                            .unwrap();
                        if let Ok(port) = port.parse::<u16>() {
                            let data: &mut Ptr<Config> = x.user_data().unwrap();
                            data.binds.push((host.to_string(), port));
                            let state = data.binds.clone();
                            x.call_on_name(
                                "bindings",
                                |l: &mut LinearLayout| {
                                    iterate_layout(l, &state);
                                },
                            );
                            x.pop_layer();
                            x.add_layer(
                                Dialog::around(TextView::new("Successfully updated!"))
                                    .title("Successful")
                                    .dismiss_button("Ok"),
                            );
                        } else {
                            x.add_layer(
                                Dialog::around(TextView::new("Invalid Port Provided"))
                                    .title("Error")
                                    .dismiss_button("Ok"),
                            );
                        }
                    },
                )
                .dismiss_button("Cancel")
        }
        fn gen_cnt(s: Ptr<Config>) -> NamedView<LinearLayout> {
            if s.binds.is_empty() {
                LinearLayout::vertical()
                    .child(TextView::new("No bindings detected"))
                    .with_name("bindings")
            } else {
                let mut layout = LinearLayout::vertical();
                iterate_layout(&mut layout, &s.binds);
                layout.with_name("bindings")
            }
        }
        fn iterate_layout(l: &mut LinearLayout, binds: &[(String, u16)]) {
            l.clear();
            if binds.is_empty() {
                l.add_child(TextView::new("No bindings detected"));
            } else {
                l.add_child(
                    LinearLayout::horizontal()
                        .child(
                            TextView::new("SNo")
                                .style(Style::merge(&[Effect::Dim.into()]))
                                .fixed_width(5),
                        )
                        .child(
                            TextView::new("Hostname")
                                .style(Style::merge(&[Effect::Dim.into()]))
                                .full_width(),
                        )
                        .child(
                            TextView::new("Port ")
                                .style(Style::merge(&[Effect::Dim.into()]))
                                .fixed_width(5),
                        )
                        .child(
                            TextView::new("")
                                .style(Style::merge(&[Effect::Dim.into()]))
                                .fixed_width(12),
                        ),
                );
            }
            binds
                .iter()
                .enumerate()
                .for_each(|(index, (host, port))| {
                    l.add_child(layout_child(index, host, port));
                });
        }
        fn layout_child(index: usize, host: &str, port: &u16) -> LinearLayout {
            LinearLayout::horizontal()
                .child(
                    TextView::new(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(format_args!("{0}.", index + 1))
                            }),
                        )
                        .align(Align::center_left())
                        .fixed_width(5),
                )
                .child(TextView::new(host).full_width())
                .child(
                    TextView::new(port.to_string()).align(Align::center()).fixed_width(5),
                )
                .child(
                    Button::new_raw(
                            "✕ Remove",
                            move |x| {
                                x.with_user_data(|x: &mut Ptr<Config>| {
                                    x.binds.remove(index);
                                });
                                let state: &mut Ptr<Config> = x.user_data().unwrap();
                                let state = state.binds.clone();
                                x.call_on_name(
                                    "bindings",
                                    |l: &mut LinearLayout| {
                                        iterate_layout(l, &state);
                                    },
                                );
                            },
                        )
                        .fixed_width(12),
                )
        }
    }
    mod dbconf {
        use cursive::{
            Cursive, View, align::HAlign, theme::{Effect, Style},
            view::{Nameable, Resizable},
            views::{
                Button, Dialog, DummyView, EditView, LinearLayout, NamedView, ScrollView,
                TextView,
            },
        };
        use crate::{
            auth::argon::{self, server::verify_server_pass},
            structs::{Config, db::{AuthDbConfig, CacheConfig, TlsConfig}},
            ui::Ptr,
        };
        pub fn db_page(s: Ptr<Config>) -> NamedView<impl View> {
            let mut layout = LinearLayout::vertical();
            render(&mut layout, s);
            ScrollView::new(layout.with_name("dbconf"))
                .show_scrollbars(true)
                .with_name("㏈ Database")
        }
        pub fn render(layout: &mut LinearLayout, s: Ptr<Config>) {
            layout.clear();
            layout.add_child(DummyView::new().fixed_height(1));
            layout
                .add_child(
                    TextView::new(
                            "All the secrets are safely encrypted using your server administrator password",
                        )
                        .style(Style::from(Effect::Dim))
                        .h_align(HAlign::Center),
                );
            layout.add_child(DummyView::new().fixed_height(2));
            layout
                .add_child(
                    TextView::new("㏈ Authentication Store Database")
                        .style(Style::from(Effect::Underline)),
                );
            match &s.database.authdb {
                AuthDbConfig::Moka { .. } => {
                    layout.add_child(authdb_moka());
                }
                AuthDbConfig::Mongodb { .. } => {
                    layout.add_child(authdb_mongodb());
                }
                AuthDbConfig::Tikv { timeout_secs, tls_config, .. } => {
                    layout.add_child(authdb_tikv(*timeout_secs, tls_config.is_some()));
                }
            }
            layout.add_child(DummyView::new().fixed_height(1));
            layout
                .add_child(
                    TextView::new("㏈ Cache Store Database")
                        .style(Style::from(Effect::Underline)),
                );
            match &s.database.cache {
                CacheConfig::Redis { .. } => {
                    layout.add_child(cache_redis());
                }
                CacheConfig::Moka => {
                    layout.add_child(cache_moka());
                }
            }
        }
        fn select_db(x: &mut Cursive) {
            x.add_layer(
                Dialog::around(
                        ScrollView::new(
                            LinearLayout::vertical()
                                .child(
                                    TextView::new("Standard DBs")
                                        .style(Style::from(Effect::Underline)),
                                )
                                .child(
                                    LinearLayout::horizontal()
                                        .child(
                                            Button::new_raw(
                                                    "TiKV (Recommended)",
                                                    |x| {
                                                        x.pop_layer();
                                                        let mut state = x
                                                            .user_data::<Ptr<Config>>()
                                                            .unwrap()
                                                            .clone();
                                                        state.database.authdb = AuthDbConfig::Tikv {
                                                            endpoints: Default::default(),
                                                            tls_config: Default::default(),
                                                            timeout_secs: 30,
                                                        };
                                                        x.call_on_name(
                                                            "dbconf",
                                                            move |x: &mut LinearLayout| {
                                                                render(x, state);
                                                            },
                                                        );
                                                    },
                                                )
                                                .fixed_width(18),
                                        )
                                        .child(DummyView::new().full_width()),
                                )
                                .child(
                                    LinearLayout::horizontal()
                                        .child(
                                            Button::new_raw(
                                                    "MongoDB (Simple Setup)",
                                                    |x| {
                                                        x.pop_layer();
                                                        let mut state = x
                                                            .user_data::<Ptr<Config>>()
                                                            .unwrap()
                                                            .clone();
                                                        state.database.authdb = AuthDbConfig::Mongodb {
                                                            url: Default::default(),
                                                        };
                                                        x.call_on_name(
                                                            "dbconf",
                                                            move |x: &mut LinearLayout| {
                                                                render(x, state);
                                                            },
                                                        );
                                                    },
                                                )
                                                .fixed_width(22),
                                        )
                                        .child(DummyView::new().full_width()),
                                )
                                .child(DummyView::new().fixed_height(1))
                                .child(
                                    TextView::new("Fake DBs")
                                        .style(Style::from(Effect::Underline)),
                                )
                                .child(
                                    TextView::new(
                                            "These databases have no persistence nor are suitable for production environment. The database are stored in RAM temporarily only for testing purposes. Never use it in production.",
                                        )
                                        .style(Style::from(Effect::Dim)),
                                )
                                .child(DummyView::new().fixed_height(1))
                                .child(
                                    LinearLayout::horizontal()
                                        .child(
                                            Button::new_raw(
                                                    "Moka",
                                                    |x| {
                                                        x.pop_layer();
                                                        let mut state = x
                                                            .user_data::<Ptr<Config>>()
                                                            .unwrap()
                                                            .clone();
                                                        state.database.authdb = AuthDbConfig::Moka {};
                                                        x.call_on_name(
                                                            "dbconf",
                                                            move |x: &mut LinearLayout| {
                                                                render(x, state);
                                                            },
                                                        );
                                                    },
                                                )
                                                .fixed_width(4),
                                        )
                                        .child(DummyView::new().full_width()),
                                ),
                        ),
                    )
                    .dismiss_button("Cancel")
                    .title("Choose your Auth Database")
                    .min_width(32)
                    .max_width(48),
            );
        }
        fn select_cache_db(x: &mut Cursive) {
            x.add_layer(
                Dialog::around(
                        ScrollView::new(
                            LinearLayout::vertical()
                                .child(
                                    LinearLayout::horizontal()
                                        .child(
                                            Button::new_raw(
                                                    "Redis (Recommended)",
                                                    |x| {
                                                        x.pop_layer();
                                                        let mut state = x
                                                            .user_data::<Ptr<Config>>()
                                                            .unwrap()
                                                            .clone();
                                                        state.database.cache = CacheConfig::Redis {
                                                            url: Default::default(),
                                                        };
                                                        x.call_on_name(
                                                            "dbconf",
                                                            move |x: &mut LinearLayout| {
                                                                render(x, state);
                                                            },
                                                        );
                                                    },
                                                )
                                                .fixed_width(19),
                                        )
                                        .child(DummyView::new().full_width()),
                                )
                                .child(
                                    LinearLayout::horizontal()
                                        .child(
                                            Button::new_raw(
                                                    "Moka (In-RAM; Best for single server setup)",
                                                    |x| {
                                                        x.pop_layer();
                                                        let mut state = x
                                                            .user_data::<Ptr<Config>>()
                                                            .unwrap()
                                                            .clone();
                                                        state.database.cache = CacheConfig::Moka;
                                                        x.call_on_name(
                                                            "dbconf",
                                                            move |x: &mut LinearLayout| {
                                                                render(x, state);
                                                            },
                                                        );
                                                    },
                                                )
                                                .fixed_width(43),
                                        )
                                        .child(DummyView::new().full_width()),
                                ),
                        ),
                    )
                    .dismiss_button("Cancel")
                    .title("Choose your Auth Database")
                    .min_width(32)
                    .max_width(48),
            );
        }
        pub fn authdb_moka() -> impl View {
            LinearLayout::horizontal()
                .child(TextView::new("㏈ Database").full_width())
                .child(
                    Button::new_raw(
                        "Moka",
                        |x| {
                            select_db(x);
                        },
                    ),
                )
        }
        pub fn authdb_mongodb() -> impl View {
            let db = LinearLayout::horizontal()
                .child(TextView::new("㏈ Database").full_width())
                .child(
                    Button::new_raw(
                        "Mongodb",
                        |x| {
                            select_db(x);
                        },
                    ),
                );
            let url = LinearLayout::horizontal()
                .child(TextView::new("🖳 Mongodb URL").full_width())
                .child(
                    Button::new_raw(
                        "Set ↗",
                        |x| {
                            set_url(x, Db::Mongo);
                        },
                    ),
                );
            LinearLayout::vertical().child(db).child(url)
        }
        pub fn cache_redis() -> impl View {
            let db = LinearLayout::horizontal()
                .child(TextView::new("㏈ Database").full_width())
                .child(
                    Button::new_raw(
                        "Redis",
                        |x| {
                            select_cache_db(x);
                        },
                    ),
                );
            let url = LinearLayout::horizontal()
                .child(TextView::new("🖳 Redis URL").full_width())
                .child(
                    Button::new_raw(
                        "Set ↗",
                        |x| {
                            set_url(x, Db::Redis);
                        },
                    ),
                );
            LinearLayout::vertical().child(db).child(url)
        }
        pub fn cache_moka() -> impl View {
            let db = LinearLayout::horizontal()
                .child(TextView::new("㏈ Database").full_width())
                .child(
                    Button::new_raw(
                        "Moka",
                        |x| {
                            select_cache_db(x);
                        },
                    ),
                );
            LinearLayout::vertical().child(db)
        }
        enum Db {
            Mongo,
            Redis,
        }
        fn set_url(x: &mut Cursive, db: Db) {
            x.add_layer(
                Dialog::around(
                        LinearLayout::vertical()
                            .child(TextView::new("Enter your server admin password"))
                            .child(EditView::new().secret().with_name("serverpass"))
                            .child(TextView::new("Enter the new url"))
                            .child(EditView::new().with_name("url")),
                    )
                    .button(
                        "Set",
                        move |x| {
                            let pass = x
                                .call_on_name(
                                    "serverpass",
                                    |x: &mut EditView| x.get_content(),
                                )
                                .unwrap();
                            let given_url = x
                                .call_on_name("url", |x: &mut EditView| x.get_content())
                                .unwrap();
                            let mut user = x.user_data::<Ptr<Config>>().unwrap().clone();
                            if !verify_server_pass(
                                    &pass,
                                    user.admin_pass_hash.as_ref().unwrap(),
                                )
                                .unwrap_or(false)
                            {
                                x.add_layer(
                                    Dialog::around(TextView::new("Invalid password"))
                                        .dismiss_button("Ok"),
                                );
                                return;
                            }
                            match db {
                                Db::Mongo => {
                                    let AuthDbConfig::Mongodb { url } = &mut user
                                        .database
                                        .authdb else {
                                        ::core::panicking::panic(
                                            "internal error: entered unreachable code",
                                        )
                                    };
                                    *url = argon::encrypt_with_key(&pass, &given_url)
                                        .into_boxed_str();
                                }
                                Db::Redis => {
                                    let CacheConfig::Redis { url } = &mut user.database.cache
                                    else {
                                        ::core::panicking::panic(
                                            "internal error: entered unreachable code",
                                        )
                                    };
                                    *url = argon::encrypt_with_key(&pass, &given_url)
                                        .into_boxed_str();
                                }
                            }
                            x.pop_layer();
                        },
                    )
                    .dismiss_button("Cancel"),
            );
        }
        enum CallNext {
            Endpoints,
            TLSConf,
        }
        mod tls {
            use std::{sync::Arc, thread::spawn};
            use cursive::{
                Cursive, align::HAlign, theme::{Effect, Style},
                view::{Nameable, Resizable},
                views::{
                    Button, Dialog, DummyView, EditView, LinearLayout, ScrollView,
                    TextView,
                },
            };
            use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};
            use crate::{
                auth::argon::{decrypt_with_key, encrypt_with_key},
                structs::{Config, db::AuthDbConfig},
                ui::Ptr,
            };
            struct Tls {
                ca_path: String,
                cert_path: String,
                key_path: String,
            }
            #[automatically_derived]
            impl ::core::fmt::Debug for Tls {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "Tls",
                        "ca_path",
                        &self.ca_path,
                        "cert_path",
                        &self.cert_path,
                        "key_path",
                        &&self.key_path,
                    )
                }
            }
            impl ::zeroize::Zeroize for Tls {
                fn zeroize(&mut self) {
                    match self {
                        #[allow(unused_variables)]
                        Tls { ca_path, cert_path, key_path } => {
                            ca_path.zeroize();
                            cert_path.zeroize();
                            key_path.zeroize()
                        }
                        _ => {}
                    }
                }
            }
            impl Drop for Tls {
                fn drop(&mut self) {
                    use ::zeroize::__internal::AssertZeroize;
                    use ::zeroize::__internal::AssertZeroizeOnDrop;
                    match self {
                        #[allow(unused_variables)]
                        Tls { ca_path, cert_path, key_path } => {
                            ca_path.zeroize_or_on_drop();
                            cert_path.zeroize_or_on_drop();
                            key_path.zeroize_or_on_drop()
                        }
                        _ => {}
                    }
                }
            }
            #[doc(hidden)]
            impl ::zeroize::ZeroizeOnDrop for Tls {}
            pub fn tls(pass: String, x: &mut Cursive) {
                x.add_layer(Dialog::around(TextView::new("Decrypting...")));
                let sink = x.cb_sink().clone();
                let conf = x.user_data::<Ptr<Config>>().unwrap().clone();
                let pass: Arc<str> = Arc::from(pass);
                spawn(move || {
                    let AuthDbConfig::Tikv { tls_config, .. } = &conf.database.authdb
                    else {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        );
                    };
                    let tls = tls_config.as_ref().unwrap();
                    let decrypted = Tls {
                        ca_path: decrypt_with_key(&pass, &tls.ca_path),
                        cert_path: decrypt_with_key(&pass, &tls.cert_path),
                        key_path: decrypt_with_key(&pass, &tls.key_path),
                    };
                    let decrypted = Zeroizing::new(decrypted);
                    _ = sink
                        .send(
                            Box::new(move |x| {
                                x.pop_layer();
                                render(x, pass.clone(), decrypted);
                            }),
                        );
                });
            }
            fn render(x: &mut Cursive, pass: Arc<str>, decrypted: Zeroizing<Tls>) {
                let p1 = pass.clone();
                let p2 = pass.clone();
                let p3 = pass.clone();
                let layout = LinearLayout::vertical()
                    .child(DummyView::new().fixed_height(1))
                    .child(
                        TextView::new(
                                "Please refer to TiKV official docs at https://tikv.org/docs/4.0/tasks/configure/security/",
                            )
                            .h_align(HAlign::Center),
                    )
                    .child(DummyView::new().fixed_height(1))
                    .child(
                        TextView::new("CA Cert").style(Style::from(Effect::Underline)),
                    )
                    .child(
                        TextView::new(
                                "The path to the file that contains the PEM encoding of the server’s CA certificates.",
                            )
                            .style(Style::from(Effect::Dim)),
                    )
                    .child(DummyView::new().fixed_height(1))
                    .child(
                        TextView::new(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("Currently \"{0}\"", decrypted.ca_path),
                                    )
                                }),
                            )
                            .with_name("ca_path"),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(DummyView::new().full_width())
                            .child(
                                Button::new_raw(
                                        "Change ↗",
                                        move |x| {
                                            set(x, p1.clone(), ToChange::CaPath);
                                        },
                                    )
                                    .fixed_width(6),
                            ),
                    )
                    .child(DummyView::new().fixed_height(1))
                    .child(
                        TextView::new("Cert Path").style(Style::from(Effect::Underline)),
                    )
                    .child(
                        TextView::new(
                                "The path to the file that contains the PEM encoding of the server’s certificate chain.",
                            )
                            .style(Style::from(Effect::Dim)),
                    )
                    .child(DummyView::new().fixed_height(1))
                    .child(
                        TextView::new(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("Currently \"{0}\"", decrypted.cert_path),
                                    )
                                }),
                            )
                            .with_name("cert_path"),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(DummyView::new().full_width())
                            .child(
                                Button::new_raw(
                                        "Change ↗",
                                        move |x| {
                                            set(x, p2.clone(), ToChange::CertPath);
                                        },
                                    )
                                    .fixed_width(6),
                            ),
                    )
                    .child(DummyView::new().fixed_height(1))
                    .child(
                        TextView::new("Key Path").style(Style::from(Effect::Underline)),
                    )
                    .child(
                        TextView::new(
                                "The path to the file that contains the PEM encoding of the server’s private key.",
                            )
                            .style(Style::from(Effect::Dim)),
                    )
                    .child(DummyView::new().fixed_height(1))
                    .child(
                        TextView::new(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("Currently \"{0}\"", decrypted.key_path),
                                    )
                                }),
                            )
                            .with_name("key_path"),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(DummyView::new().full_width())
                            .child(
                                Button::new_raw(
                                        "Change ↗",
                                        move |x| {
                                            set(x, p3.clone(), ToChange::KeyPath);
                                        },
                                    )
                                    .fixed_width(6),
                            ),
                    );
                x.add_layer(
                    Dialog::around(ScrollView::new(layout))
                        .title("Configure TLS")
                        .dismiss_button("Back")
                        .full_screen(),
                )
            }
            enum ToChange {
                CaPath,
                CertPath,
                KeyPath,
            }
            fn set(x: &mut Cursive, pass: Arc<str>, ty: ToChange) {
                x.add_layer(
                    Dialog::around(
                            LinearLayout::vertical()
                                .child(TextView::new("Enter new path"))
                                .child(EditView::new().with_name("path_data")),
                        )
                        .title("Set Path")
                        .button(
                            "Ok",
                            move |x| {
                                let pass = pass.clone();
                                let path = x
                                    .call_on_name(
                                        "path_data",
                                        |x: &mut EditView| x.get_content(),
                                    )
                                    .unwrap();
                                let mut conf = x
                                    .user_data::<Ptr<Config>>()
                                    .unwrap()
                                    .clone();
                                let AuthDbConfig::Tikv { tls_config, .. } = &mut conf
                                    .database
                                    .authdb else {
                                    ::core::panicking::panic(
                                        "internal error: entered unreachable code",
                                    );
                                };
                                let tls = tls_config.as_mut().unwrap();
                                let name = match ty {
                                    ToChange::CaPath => {
                                        tls.ca_path = encrypt_with_key(&pass, &path)
                                            .into_boxed_str();
                                        "ca_path"
                                    }
                                    ToChange::CertPath => {
                                        tls.cert_path = encrypt_with_key(&pass, &path)
                                            .into_boxed_str();
                                        "cert_path"
                                    }
                                    ToChange::KeyPath => {
                                        tls.key_path = encrypt_with_key(&pass, &path)
                                            .into_boxed_str();
                                        "key_path"
                                    }
                                };
                                x.call_on_name(
                                    name,
                                    move |x: &mut TextView| {
                                        x.set_content(
                                            ::alloc::__export::must_use({
                                                ::alloc::fmt::format(
                                                    format_args!("Currently \"{0}\"", &path as &str),
                                                )
                                            }),
                                        );
                                    },
                                );
                            },
                        )
                        .dismiss_button("Cancel")
                        .min_width(32)
                        .max_width(64),
                );
            }
        }
        mod url {
            use std::{
                sync::{Arc, Mutex},
                thread::spawn,
            };
            use cursive::{
                Cursive, theme::{Effect, Style},
                view::{Nameable, Resizable},
                views::{
                    Button, Dialog, DummyView, EditView, LinearLayout, ScrollView,
                    TextView,
                },
            };
            use zeroize::Zeroizing;
            use crate::{
                auth::argon::{decrypt_with_key, encrypt_with_key},
                structs::{Config, db::AuthDbConfig},
                ui::Ptr,
            };
            type DecryptedUrls = Arc<Mutex<Zeroizing<Vec<Box<str>>>>>;
            pub fn url(pass: String, x: &mut Cursive) {
                x.add_layer(Dialog::around(TextView::new("Decrypting...")));
                let sink = x.cb_sink().clone();
                let conf = x.user_data::<Ptr<Config>>().unwrap().clone();
                let pass: Arc<str> = Arc::from(pass);
                spawn(move || {
                    let AuthDbConfig::Tikv { endpoints, .. } = &conf.database.authdb
                    else {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        );
                    };
                    let decrypted = endpoints
                        .iter()
                        .map(|x| decrypt_with_key(&pass, &x as &str).into_boxed_str())
                        .collect::<Vec<_>>();
                    let decrypted = Arc::new(Mutex::new(Zeroizing::new(decrypted)));
                    _ = sink
                        .send(
                            Box::new(move |x| {
                                x.pop_layer();
                                render(x, pass.clone(), decrypted);
                            }),
                        );
                });
            }
            pub fn render(x: &mut Cursive, pass: Arc<str>, decrypted: DecryptedUrls) {
                let mut layout = LinearLayout::vertical();
                render_ui(&mut layout, decrypted.clone());
                let decr = decrypted.clone();
                x.add_layer(
                    Dialog::around(ScrollView::new(layout.with_name("tikv_hostnames")))
                        .title("Configure hostnames")
                        .button(
                            "New",
                            move |x| {
                                add_new(x, pass.clone(), decr.clone());
                            },
                        )
                        .dismiss_button("Back")
                        .full_screen(),
                )
            }
            fn add_new(x: &mut Cursive, pass: Arc<str>, decr: DecryptedUrls) {
                x.add_layer(
                    Dialog::around(
                            LinearLayout::vertical()
                                .child(TextView::new("Enter the hostname"))
                                .child(EditView::new().with_name("hostname")),
                        )
                        .button(
                            "Add",
                            move |x| {
                                let hostname = x
                                    .call_on_name(
                                        "hostname",
                                        |x: &mut EditView| x.get_content(),
                                    )
                                    .unwrap();
                                let encrypted = encrypt_with_key(&pass, &hostname);
                                let conf = x.user_data::<Ptr<Config>>().unwrap();
                                let AuthDbConfig::Tikv { endpoints, .. } = &mut conf
                                    .database
                                    .authdb else {
                                    ::core::panicking::panic(
                                        "internal error: entered unreachable code",
                                    );
                                };
                                let mut vect = endpoints.to_vec();
                                vect.push(encrypted.clone().into_boxed_str());
                                *endpoints = vect.into_boxed_slice();
                                decr.lock()
                                    .map_or_else(|x| x.into_inner(), |x| x)
                                    .push(hostname.to_string().into_boxed_str());
                                let decr = decr.clone();
                                x.pop_layer();
                                x.call_on_name(
                                    "tikv_hostnames",
                                    move |layout: &mut LinearLayout| {
                                        render_ui(layout, decr.clone());
                                    },
                                );
                            },
                        )
                        .dismiss_button("Cancel")
                        .title("Add Hostname")
                        .min_width(32)
                        .max_width(64),
                );
            }
            pub fn render_ui(layout: &mut LinearLayout, decrypted: DecryptedUrls) {
                layout.clear();
                layout
                    .add_child(
                        LinearLayout::horizontal()
                            .child(
                                TextView::new("Model ID")
                                    .style(Style::merge(&[Effect::Dim.into()]))
                                    .full_width(),
                            )
                            .child(
                                TextView::new("Actions")
                                    .style(Style::merge(&[Effect::Dim.into()]))
                                    .fixed_width(10),
                            ),
                    );
                let decr2 = decrypted.clone();
                decrypted
                    .lock()
                    .map_or_else(|e| e.into_inner(), |v| v)
                    .iter()
                    .enumerate()
                    .for_each(move |(index, x)| {
                        let decr2 = decr2.clone();
                        layout
                            .add_child(
                                LinearLayout::horizontal()
                                    .child(TextView::new(x as &str).full_width())
                                    .child(
                                        Button::new_raw(
                                                "Remove",
                                                move |x| {
                                                    let decr = decr2.clone();
                                                    let conf = x.user_data::<Ptr<Config>>().unwrap();
                                                    let AuthDbConfig::Tikv { endpoints, .. } = &mut conf
                                                        .database
                                                        .authdb else {
                                                        ::core::panicking::panic(
                                                            "internal error: entered unreachable code",
                                                        );
                                                    };
                                                    let mut vect = endpoints.to_vec();
                                                    vect.remove(index);
                                                    decr.lock()
                                                        .map_or_else(|x| x.into_inner(), |x| x)
                                                        .remove(index);
                                                    *endpoints = vect.into_boxed_slice();
                                                    x.call_on_name(
                                                        "tikv_hostnames",
                                                        move |layout: &mut LinearLayout| {
                                                            render_ui(layout, decr.clone());
                                                        },
                                                    );
                                                },
                                            )
                                            .fixed_width(6),
                                    )
                                    .child(DummyView::new().fixed_width(4)),
                            );
                    });
            }
        }
        fn get_admin_pass(x: &mut Cursive, tocallnext: CallNext) {
            x.add_layer(
                Dialog::around(
                        LinearLayout::vertical()
                            .child(
                                TextView::new("Please enter your administrator password"),
                            )
                            .child(EditView::new().secret().with_name("admin_pass")),
                    )
                    .title("Authentication Required")
                    .button(
                        "Continue",
                        move |x| {
                            let pass = x
                                .call_on_name(
                                    "admin_pass",
                                    |x: &mut EditView| x.get_content(),
                                )
                                .unwrap();
                            let hash: &str = x
                                .user_data::<Ptr<Config>>()
                                .unwrap()
                                .admin_pass_hash
                                .as_ref()
                                .unwrap();
                            if !verify_server_pass(&pass, hash).unwrap_or(false) {
                                x.add_layer(
                                    Dialog::around(TextView::new("Invalid Password"))
                                        .dismiss_button("Okay"),
                                );
                                return;
                            }
                            x.pop_layer();
                            let password = pass.to_string();
                            match tocallnext {
                                CallNext::Endpoints => {
                                    url::url(password, x);
                                }
                                CallNext::TLSConf => {
                                    tls::tls(password, x);
                                }
                            }
                        },
                    )
                    .dismiss_button("Cancel"),
            );
        }
        fn update_tls(x: &mut Cursive) {
            x.add_layer(
                Dialog::around(
                        LinearLayout::vertical()
                            .child(TextView::new("Select the state"))
                            .child(
                                LinearLayout::horizontal()
                                    .child(
                                        Button::new_raw(
                                            "Enabled",
                                            |x| {
                                                x.pop_layer();
                                                let mut state = x
                                                    .user_data::<Ptr<Config>>()
                                                    .unwrap()
                                                    .clone();
                                                let AuthDbConfig::Tikv { tls_config, .. } = &mut state
                                                    .database
                                                    .authdb else {
                                                    ::core::panicking::panic(
                                                        "internal error: entered unreachable code",
                                                    )
                                                };
                                                *tls_config = Some(TlsConfig::default());
                                                x.call_on_name(
                                                    "dbconf",
                                                    move |x: &mut LinearLayout| {
                                                        render(x, state);
                                                    },
                                                );
                                            },
                                        ),
                                    )
                                    .child(DummyView::new().full_width()),
                            )
                            .child(
                                LinearLayout::horizontal()
                                    .child(
                                        Button::new_raw(
                                            "Disabled",
                                            |x| {
                                                x.pop_layer();
                                                let mut state = x
                                                    .user_data::<Ptr<Config>>()
                                                    .unwrap()
                                                    .clone();
                                                let AuthDbConfig::Tikv { tls_config, .. } = &mut state
                                                    .database
                                                    .authdb else {
                                                    ::core::panicking::panic(
                                                        "internal error: entered unreachable code",
                                                    )
                                                };
                                                *tls_config = None;
                                                x.call_on_name(
                                                    "dbconf",
                                                    move |x: &mut LinearLayout| {
                                                        render(x, state);
                                                    },
                                                );
                                            },
                                        ),
                                    )
                                    .child(DummyView::new().full_width()),
                            ),
                    )
                    .dismiss_button("Cancel")
                    .min_width(32)
                    .max_width(64),
            );
        }
        pub fn authdb_tikv(timeout: u64, tls_enabled: bool) -> impl View {
            let db = LinearLayout::horizontal()
                .child(TextView::new("㏈ Database").full_width())
                .child(
                    Button::new_raw(
                        "TiKV",
                        |x| {
                            select_db(x);
                        },
                    ),
                );
            let url = LinearLayout::horizontal()
                .child(TextView::new("🖳 TiPD Endpoints").full_width())
                .child(
                    Button::new_raw(
                        "Configure ↗",
                        |x| {
                            get_admin_pass(x, CallNext::Endpoints);
                        },
                    ),
                );
            let tls = LinearLayout::horizontal()
                .child(TextView::new("🖧 TLS").full_width())
                .child(
                    Button::new_raw(
                        if tls_enabled { "Enabled ↗" } else { "Disabled ↗" },
                        |x| {
                            update_tls(x);
                        },
                    ),
                );
            let tls_conf = LinearLayout::horizontal()
                .child(TextView::new("🖥 TLS Settings").full_width())
                .child(
                    Button::new_raw(
                        "Configure ↗",
                        |x| {
                            get_admin_pass(x, CallNext::TLSConf);
                        },
                    ),
                );
            let timeout = LinearLayout::horizontal()
                .child(TextView::new("🖧 Timeout (in seconds)").full_width())
                .child(
                    Button::new_raw(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(format_args!("<{0}>", timeout))
                            }),
                            |x| {
                                x.add_layer(
                                    Dialog::around(
                                            EditView::new()
                                                .on_edit(|x, i, _| {
                                                    let mut state = x
                                                        .user_data::<Ptr<Config>>()
                                                        .unwrap()
                                                        .clone();
                                                    if let Ok(data) = i.parse::<u64>() {
                                                        let AuthDbConfig::Tikv { timeout_secs, .. } = &mut state
                                                            .database
                                                            .authdb else {
                                                            ::core::panicking::panic(
                                                                "internal error: entered unreachable code",
                                                            )
                                                        };
                                                        *timeout_secs = data;
                                                        x.call_on_name(
                                                            "tikv_timeout",
                                                            move |btn: &mut Button| {
                                                                btn.set_label_raw(
                                                                    ::alloc::__export::must_use({
                                                                        ::alloc::fmt::format(format_args!("<{0}>", data))
                                                                    }),
                                                                );
                                                            },
                                                        );
                                                    }
                                                }),
                                        )
                                        .dismiss_button("Okay")
                                        .title("Enter new timeout")
                                        .min_width(32)
                                        .max_width(48),
                                );
                            },
                        )
                        .with_name("tikv_timeout"),
                );
            let mut out = LinearLayout::vertical()
                .child(db)
                .child(url)
                .child(timeout)
                .child(DummyView::new().fixed_height(1))
                .child(TextView::new("🖧 TLS").style(Style::from(Effect::Underline)))
                .child(tls);
            if tls_enabled {
                out = out.child(tls_conf);
            }
            out
        }
    }
    mod llama {
        use cursive::{
            theme::{Effect, Style},
            view::{Nameable, Resizable},
            views::{Button, DummyView, LinearLayout, NamedView, ScrollView, TextView},
        };
        use crate::{structs::Config, ui::Ptr};
        mod manager {
            use std::sync::Arc;
            use crate::{
                auth::{
                    argon::{encrypt_with_key, server::verify_server_pass},
                    gen_uid,
                },
                structs::{Capabilities, Config, LlamaServer, ModelFlag},
                ui::Ptr,
            };
            use cursive::{
                With, theme::{Effect, Style},
                utils::markup::StyledString, view::{Margins, Nameable, Resizable},
                views::{
                    Button, Checkbox, Dialog, DummyView, EditView, LinearLayout,
                    ResizedView, ScrollView, TextView,
                },
            };
            pub fn launch(data: Ptr<Config>) -> ResizedView<Dialog> {
                let s1 = data.clone();
                let mut layout = LinearLayout::vertical();
                render_table(&mut layout, data.clone());
                Dialog::new()
                    .content(
                        ScrollView::new(layout.with_name("renderedtable"))
                            .show_scrollbars(true),
                    )
                    .button(
                        "New",
                        move |s| {
                            if let Some(_) = s1.admin_pass_hash {
                                s.add_layer(new_server(s1.clone(), None));
                            } else {
                                s.add_layer(
                                    Dialog::around(
                                            TextView::new("Please set a server password first!"),
                                        )
                                        .dismiss_button("Ok"),
                                );
                            }
                        },
                    )
                    .dismiss_button("Back")
                    .full_screen()
            }
            pub fn render_table(layout: &mut LinearLayout, conf: Ptr<Config>) {
                layout.clear();
                layout
                    .add_child(
                        LinearLayout::horizontal()
                            .child(
                                TextView::new("Model ID")
                                    .style(Style::merge(&[Effect::Dim.into()]))
                                    .fixed_width(34),
                            )
                            .child(
                                TextView::new("Model Name")
                                    .style(Style::merge(&[Effect::Dim.into()]))
                                    .full_width(),
                            )
                            .child(
                                TextView::new("Model URL")
                                    .style(Style::merge(&[Effect::Dim.into()]))
                                    .full_width(),
                            )
                            .child(
                                Button::new_raw(
                                        StyledString::styled("Feat (i)", Style::from(Effect::Dim)),
                                        |x| {
                                            x.add_layer(
                                                Dialog::around(
                                                        ScrollView::new(
                                                            LinearLayout::vertical()
                                                                .child(TextView::new("`Abbr` : Capability"))
                                                                .child(TextView::new("`A`    : Audio"))
                                                                .child(TextView::new("`I`    : Image"))
                                                                .child(TextView::new("`F`    : Files")),
                                                        ),
                                                    )
                                                    .title("Legend")
                                                    .dismiss_button("Got it"),
                                            );
                                        },
                                    )
                                    .fixed_width(10),
                            )
                            .child(DummyView::new().fixed_width(3))
                            .child(
                                TextView::new("Actions")
                                    .style(Style::merge(&[Effect::Dim.into()]))
                                    .fixed_width(7),
                            ),
                    );
                for (k, v) in &conf.llama.models {
                    let conf2 = conf.clone();
                    let key = Some(k.to_string());
                    let id = k as &str;
                    let name = &v.name as &str;
                    let url = &v.url as &str;
                    let mut cap = <[_]>::into_vec(::alloc::boxed::box_new([" "]));
                    if v.capabilities.has(ModelFlag::Audio) {
                        cap.push("A");
                    }
                    if v.capabilities.has(ModelFlag::Image) {
                        cap.push("I");
                    }
                    if v.capabilities.has(ModelFlag::Files) {
                        cap.push("F");
                    }
                    let cap = cap.join("");
                    layout
                        .add_child(
                            LinearLayout::horizontal()
                                .child(TextView::new(id).fixed_width(34))
                                .child(TextView::new(name).full_width())
                                .child(TextView::new(url).full_width())
                                .child(TextView::new(cap).fixed_width(10))
                                .child(DummyView::new().fixed_width(6))
                                .child(
                                    Button::new_raw(
                                            "Show",
                                            move |x| {
                                                let conf3 = conf2.clone();
                                                let conf4 = conf2.clone();
                                                let key_to_pass = key.clone();
                                                let key_to_pass2 = key.clone();
                                                x.add_layer(
                                                    Dialog::around(
                                                            LinearLayout::vertical()
                                                                .child(
                                                                    Button::new_raw(
                                                                        "Edit",
                                                                        move |x| {
                                                                            x.pop_layer();
                                                                            x.add_layer(new_server(conf3.clone(), key_to_pass.clone()));
                                                                        },
                                                                    ),
                                                                )
                                                                .child(
                                                                    Button::new_raw(
                                                                        "Remove",
                                                                        move |x| {
                                                                            _ = conf4
                                                                                .clone()
                                                                                .llama
                                                                                .models
                                                                                .remove(&key_to_pass2.clone().unwrap() as &str);
                                                                            x.pop_layer();
                                                                            let conf2 = conf4.clone();
                                                                            x.call_on_name(
                                                                                "renderedtable",
                                                                                move |layout: &mut LinearLayout| {
                                                                                    render_table(layout, conf2);
                                                                                },
                                                                            );
                                                                        },
                                                                    ),
                                                                ),
                                                        )
                                                        .title("Select an action")
                                                        .dismiss_button("Cancel"),
                                                );
                                            },
                                        )
                                        .fixed_width(4),
                                ),
                        );
                }
            }
            pub fn new_server(
                conf: Ptr<Config>,
                key: Option<String>,
            ) -> ResizedView<ResizedView<Dialog>> {
                let mut llama = None;
                if let Some(key) = key.as_ref() {
                    llama = conf
                        .llama
                        .models
                        .get(key as &str)
                        .map(|x| Arc::new(x.clone()));
                }
                let l1 = llama.clone();
                let l2 = llama.clone();
                let l3 = llama.clone();
                let l4 = llama.clone();
                let l5 = llama.clone();
                let l6 = llama.clone();
                let l7 = llama.clone();
                let orig_key = key;
                Dialog::new()
                    .content(
                        ScrollView::new(
                                LinearLayout::vertical()
                                    .with(move |x| {
                                        x.add_child(TextView::new("Model Name"));
                                        x.add_child(
                                            EditView::new()
                                                .with(move |x| {
                                                    if let Some(d) = l1 {
                                                        x.set_content(&d.name as &str);
                                                    }
                                                })
                                                .with_name("model_name"),
                                        );
                                        x.add_child(DummyView::new().fixed_height(1));
                                        x.add_child(
                                            TextView::new("Server Url (scheme://url:port)"),
                                        );
                                        x.add_child(
                                            EditView::new()
                                                .with(move |x| {
                                                    if let Some(d) = l2 {
                                                        x.set_content(&d.url as &str);
                                                    }
                                                })
                                                .with_name("server_url"),
                                        );
                                        x.add_child(DummyView::new().fixed_height(1));
                                        x.add_child(
                                            TextView::new("Server Admin Password (for verification)"),
                                        );
                                        x.add_child(
                                            EditView::new().secret().with_name("server_admin_key"),
                                        );
                                        x.add_child(DummyView::new().fixed_height(1));
                                        x.add_child(TextView::new("API Key (leave blank if none)"));
                                        x.add_child(
                                            EditView::new()
                                                .with(move |x| {
                                                    if let Some(data) = l3 {
                                                        if data.apikey.is_some() {
                                                            x.set_content("< unchanged >");
                                                        }
                                                    }
                                                })
                                                .with_name("api_key"),
                                        );
                                        x.add_child(DummyView::new().fixed_height(1));
                                        x.add_child(TextView::new("Model Capabilities"));
                                        x.add_child(
                                            LinearLayout::horizontal()
                                                .child(TextView::new("Image Support").full_width())
                                                .child(
                                                    Checkbox::new()
                                                        .with(move |x| {
                                                            if let Some(data) = l4 {
                                                                x.set_checked(data.capabilities.has(ModelFlag::Image));
                                                            }
                                                        })
                                                        .with_name("img"),
                                                ),
                                        );
                                        x.add_child(
                                            LinearLayout::horizontal()
                                                .child(TextView::new("Audio Support").full_width())
                                                .child(
                                                    Checkbox::new()
                                                        .with(move |x| {
                                                            if let Some(data) = l5 {
                                                                x.set_checked(data.capabilities.has(ModelFlag::Audio));
                                                            }
                                                        })
                                                        .with_name("aud"),
                                                ),
                                        );
                                        x.add_child(
                                            LinearLayout::horizontal()
                                                .child(TextView::new("Files Support").full_width())
                                                .child(
                                                    Checkbox::new()
                                                        .with(move |x| {
                                                            if let Some(data) = l6 {
                                                                x.set_checked(data.capabilities.has(ModelFlag::Files));
                                                            }
                                                        })
                                                        .with_name("file"),
                                                ),
                                        );
                                    }),
                            )
                            .show_scrollbars(true),
                    )
                    .button(
                        "Confirm",
                        move |x| {
                            let model = x
                                .call_on_name(
                                    "model_name",
                                    |x: &mut EditView| x.get_content(),
                                )
                                .unwrap();
                            let admin_pass = x
                                .call_on_name(
                                    "server_admin_key",
                                    |x: &mut EditView| x.get_content(),
                                )
                                .unwrap();
                            if !verify_server_pass(
                                    &admin_pass,
                                    conf.admin_pass_hash.as_ref().unwrap(),
                                )
                                .unwrap()
                            {
                                x.add_layer(
                                    Dialog::around(
                                            TextView::new("Invalid Sever Administrator Password"),
                                        )
                                        .dismiss_button("Ok"),
                                );
                                return;
                            }
                            let url = x
                                .call_on_name(
                                    "server_url",
                                    |x: &mut EditView| x.get_content(),
                                )
                                .unwrap();
                            let api = x
                                .call_on_name("api_key", |x: &mut EditView| x.get_content())
                                .unwrap();
                            let mut key = None;
                            if api.as_str() != "" {
                                if api.as_str() == "< unchanged >" {
                                    if let Some(x) = l7
                                        .clone()
                                        .map(|x| x.apikey.clone())
                                        .flatten()
                                    {
                                        key = Some(x);
                                    } else {
                                        key = Some(
                                            encrypt_with_key(&admin_pass, api.as_str()).into_boxed_str(),
                                        );
                                    }
                                } else {
                                    key = Some(
                                        encrypt_with_key(&admin_pass, api.as_str()).into_boxed_str(),
                                    );
                                }
                            }
                            let img = x
                                .call_on_name("img", |x: &mut Checkbox| x.is_checked())
                                .unwrap();
                            let audio = x
                                .call_on_name("aud", |x: &mut Checkbox| x.is_checked())
                                .unwrap();
                            let file = x
                                .call_on_name("file", |x: &mut Checkbox| x.is_checked())
                                .unwrap();
                            conf.clone()
                                .llama
                                .models
                                .insert(
                                    orig_key
                                        .clone()
                                        .map(|x| x.into_boxed_str())
                                        .unwrap_or(gen_uid().unwrap().into_boxed_str()),
                                    LlamaServer {
                                        name: model.to_string().into_boxed_str(),
                                        url: url.to_string().into_boxed_str(),
                                        apikey: key,
                                        capabilities: {
                                            let mut capab = Capabilities(0u16);
                                            if img {
                                                capab.add(ModelFlag::Image);
                                            }
                                            if audio {
                                                capab.add(ModelFlag::Audio);
                                            }
                                            if file {
                                                capab.add(ModelFlag::Files);
                                            }
                                            capab
                                        },
                                    },
                                );
                            x.pop_layer();
                            let conf2 = conf.clone();
                            x.call_on_name(
                                "renderedtable",
                                move |layout: &mut LinearLayout| {
                                    render_table(layout, conf2);
                                },
                            );
                        },
                    )
                    .dismiss_button("Cancel")
                    .padding(Margins::lrtb(1, 1, 1, 1))
                    .max_height(50)
                    .max_width(40)
            }
        }
        pub fn llama_page(s: Ptr<Config>) -> NamedView<ScrollView<LinearLayout>> {
            let mut layout = LinearLayout::vertical();
            layout.add_child(DummyView::new().fixed_height(1));
            layout
                .add_child(
                    TextView::new(
                        "AHQ AI uses llama-server to provide inference. You need to host a llama server with a given model that you can configure here. LLAMA.CPP allows us to enable audio, image, file support!",
                    ),
                );
            layout.add_child(DummyView::new().fixed_height(1));
            layout
                .add_child(
                    TextView::new("Models")
                        .style(Style::merge(&[Effect::Underline.into()])),
                );
            let s1 = s.clone();
            layout
                .add_child(
                    LinearLayout::horizontal()
                        .child(TextView::new("🖧 Models").full_width())
                        .child(
                            Button::new_raw(
                                "Launch Model Manager ↗",
                                move |x| {
                                    x.add_layer(manager::launch(s1.clone()));
                                },
                            ),
                        ),
                );
            ScrollView::new(layout).show_scrollbars(true).with_name("🖧 Llama Server")
        }
    }
    pub(crate) mod lazy {
        use std::sync::{Arc, Mutex};
        use cursive::view::{View, ViewWrapper};
        use cursive::{CbSink, Cursive, Printer, wrap_impl};
        use crate::structs::{Authentication, Config};
        use crate::ui::Ptr;
        pub struct OnAuthStateUpdate<V: View, F: Fn(&mut Cursive) + 'static> {
            inner: V,
            sink: CbSink,
            last_state: Arc<Mutex<Authentication>>,
            callback: F,
        }
        impl<V: View, F: Fn(&mut Cursive) + 'static> OnAuthStateUpdate<V, F> {
            pub fn new(inner: V, siv: &mut Cursive, callback: F) -> Self {
                Self {
                    inner,
                    sink: siv.cb_sink().clone(),
                    last_state: Arc::new(
                        Mutex::new({
                            let data: &mut Ptr<Config> = siv.user_data().unwrap();
                            match data.authentication {
                                Authentication::Account { .. } => Authentication::OpenToAll,
                                Authentication::OpenToAll => {
                                    Authentication::Account {
                                        registration_allowed: false,
                                        max_memory: 64,
                                        time_cost: 5,
                                    }
                                }
                            }
                        }),
                    ),
                    callback,
                }
            }
        }
        impl<V: View, T: Fn(&mut Cursive) + Send + Sync + 'static> ViewWrapper
        for OnAuthStateUpdate<V, T> {
            type V = V;
            fn with_view<F, R>(&self, f: F) -> ::std::option::Option<R>
            where
                F: ::std::ops::FnOnce(&Self::V) -> R,
            {
                ::std::option::Option::Some(f(&self.inner))
            }
            fn with_view_mut<F, R>(&mut self, f: F) -> ::std::option::Option<R>
            where
                F: ::std::ops::FnOnce(&mut Self::V) -> R,
            {
                ::std::option::Option::Some(f(&mut self.inner))
            }
            fn into_inner(self) -> ::std::result::Result<Self::V, Self>
            where
                Self::V: ::std::marker::Sized,
            {
                ::std::result::Result::Ok(self.inner)
            }
            fn wrap_draw(&self, printer: &Printer) {
                let cb_ref = &self.callback;
                let cb_ref: &'static T = unsafe { &*(cb_ref as *const T) };
                let state = self.last_state.clone();
                _ = self
                    .sink
                    .clone()
                    .send(
                        Box::new(move |x| {
                            let mut lock = state.lock().unwrap();
                            let data: &mut Ptr<Config> = x.user_data().unwrap();
                            let auth = &data.authentication;
                            if &*lock != auth {
                                *lock = auth.clone();
                                (cb_ref)(x);
                            }
                        }),
                    );
                self.inner.draw(printer);
            }
        }
    }
    pub static ASYNC: LazyLock<Runtime> = LazyLock::new(|| {
        Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Unable to build async runtime")
    });
    fn general(l: &mut LinearLayout, c_: Ptr<Config>) {
        l.add_child(
            TextView::new("Welcome to AHQ-AI Server Configuration")
                .align(Align::center())
                .style(Style::merge(&[PaletteColor::Highlight.into()]))
                .fixed_height(3),
        );
        l.add_child(
            TextView::new(
                    ::alloc::__export::must_use({
                        ::alloc::fmt::format(format_args!("AHQ AI Server v{0}", "0.3.2"))
                    }),
                )
                .align(Align::top_right())
                .style(Style::merge(&[Effect::Dim.into()]))
                .fixed_height(2),
        );
        l.add_child(
            TextView::new("Quick Guide").style(Style::merge(&[Effect::Underline.into()])),
        );
        l.add_child(TextView::new("» Use ← ↑ → ↓ to navigate"));
        l.add_child(TextView::new("» Press <Enter> key to interact with buttons"));
        l.add_child(
            TextView::new("» You can also use mouse to interact with buttons or tabs"),
        );
        l.add_child(TextView::new("» You can also scroll with the mouse scrollbar"));
        l.add_child(
            TextView::new(
                "» <q> key, <Ctrl+C> or going to <Save> tab updates the config file",
            ),
        );
        l.add_child(DummyView::new().fixed_height(1).full_width());
        l.add_child(
            TextView::new("General Settings")
                .style(Style::merge(&[Effect::Underline.into()])),
        );
        l.add_child(binds(c_.clone()));
        l.add_child(
            LinearLayout::horizontal()
                .child(TextView::new("⚒ Administrator Password").full_width())
                .child(
                    Button::new_raw(
                        "Update ↗",
                        move |x| {
                            x.add_layer(
                                Dialog::around(
                                        LinearLayout::vertical()
                                            .child(TextView::new("Old Password"))
                                            .child(EditView::new().secret().with_name("old_pwd"))
                                            .child(DummyView::new().fixed_height(1))
                                            .child(TextView::new("New Password"))
                                            .child(EditView::new().secret().with_name("new_pwd"))
                                            .child(TextView::new("Press Enter key to submit"))
                                            .child(
                                                TextView::new(
                                                    "The UI might hang for a moment due to hashing algorithm and secret migration",
                                                ),
                                            ),
                                    )
                                    .title("Change Administrator Password")
                                    .button(
                                        "Change",
                                        |x| {
                                            let old_pass = x
                                                .call_on_name("old_pwd", |x: &mut EditView| x.get_content())
                                                .unwrap();
                                            let new_pass = x
                                                .call_on_name("new_pwd", |x: &mut EditView| x.get_content())
                                                .unwrap();
                                            if old_pass.len().min(new_pass.len()) <= 8 {
                                                x.add_layer(
                                                    Dialog::around(
                                                            TextView::new("Password must have more than 8 characters"),
                                                        )
                                                        .dismiss_button("Ok"),
                                                );
                                                return;
                                            }
                                            let mut conf: Ptr<Config> = x
                                                .user_data::<Ptr<Config>>()
                                                .unwrap()
                                                .clone();
                                            if !verify_server_pass(
                                                    old_pass.as_str(),
                                                    conf.admin_pass_hash.as_ref().unwrap(),
                                                )
                                                .unwrap_or_default()
                                            {
                                                x.add_layer(
                                                    Dialog::around(TextView::new("Old password did not match"))
                                                        .dismiss_button("Ok"),
                                                );
                                                return;
                                            }
                                            conf.admin_pass_hash = Some(
                                                hash_server_pass(&new_pass).unwrap(),
                                            );
                                            migrate_config(&old_pass, &new_pass, conf.deref_mut());
                                            x.pop_layer();
                                            x.add_layer(
                                                Dialog::around(
                                                        TextView::new(
                                                            "Password change was successful, all your encrypted keys were updated.",
                                                        ),
                                                    )
                                                    .dismiss_button("Ok"),
                                            );
                                        },
                                    )
                                    .dismiss_button("Cancel"),
                            );
                        },
                    ),
                )
                .child(DummyView::new().fixed_width(2))
                .child(
                    Button::new_raw(
                        "Remove ↗",
                        move |x| {
                            x.add_layer(
                                Dialog::around(
                                        TextView::new(
                                            "This will invalidate the configuration and you will lose all of your encrypted secrets!",
                                        ),
                                    )
                                    .title("Danger")
                                    .dismiss_button("Cancel")
                                    .button(
                                        "OK, I understand the risks",
                                        |x| {
                                            let c_: &mut Ptr<Config> = x.user_data().unwrap();
                                            c_.admin_pass_hash = None;
                                            x.pop_layer();
                                        },
                                    ),
                            );
                        },
                    ),
                ),
        );
        l.add_child(
            LinearLayout::horizontal()
                .child(TextView::new("⚒ Authentication Type").full_width())
                .child(
                    Button::new_raw(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "{0} ↗",
                                        match c_.authentication {
                                            Authentication::OpenToAll => "No Auth",
                                            Authentication::Account { .. } => "Account",
                                        },
                                    ),
                                )
                            }),
                            move |x| {
                                x.add_layer(
                                    Dialog::around(
                                            SelectView::new()
                                                .item("No Auth (OpenToAll)", 0u8)
                                                .item("Account (Account)", 2u8)
                                                .on_submit(|x, bit| {
                                                    let c_: &mut Ptr<Config> = x.user_data().unwrap();
                                                    c_.authentication = match bit {
                                                        0 => Authentication::OpenToAll,
                                                        2 => {
                                                            Authentication::Account {
                                                                registration_allowed: true,
                                                                max_memory: 64,
                                                                time_cost: 5,
                                                            }
                                                        }
                                                        _ => {
                                                            ::core::panicking::panic(
                                                                "internal error: entered unreachable code",
                                                            )
                                                        }
                                                    };
                                                    let label = ::alloc::__export::must_use({
                                                        ::alloc::fmt::format(
                                                            format_args!(
                                                                "{0} ↗",
                                                                match c_.authentication {
                                                                    Authentication::OpenToAll => "No Auth",
                                                                    Authentication::Account { .. } => "Account",
                                                                },
                                                            ),
                                                        )
                                                    });
                                                    x.call_on_name(
                                                        "auth_type",
                                                        move |x: &mut Button| {
                                                            x.set_label_raw(label);
                                                        },
                                                    );
                                                    x.pop_layer();
                                                })
                                                .with_name("themeselect"),
                                        )
                                        .title("Authentication Type")
                                        .dismiss_button("Cancel"),
                                );
                            },
                        )
                        .with_name("auth_type"),
                ),
        );
        l.add_child(
            LinearLayout::horizontal()
                .child(TextView::new("🖌 TUI Theme").full_width())
                .child(
                    Button::new_raw(
                        "Select ↗",
                        move |x| {
                            x.add_layer(
                                Dialog::around(
                                        SelectView::new()
                                            .item("Default Theme", 0u8)
                                            .item("Monochrome Theme", 1u8)
                                            .on_submit(|x, bit| {
                                                x.set_theme(
                                                    match bit {
                                                        0 => Theme::retro(),
                                                        1 => Theme::terminal_default(),
                                                        _ => {
                                                            ::core::panicking::panic(
                                                                "internal error: entered unreachable code",
                                                            )
                                                        }
                                                    },
                                                );
                                                x.call_on_name(
                                                    "themeselect",
                                                    |x: &mut SelectView| { x.set_selection(*bit as usize) },
                                                );
                                                x.pop_layer();
                                                if let Some(mut home) = home_dir() {
                                                    home.push(".ahqaiservertheme");
                                                    _ = fs::write(
                                                        &home,
                                                        <[_]>::into_vec(::alloc::boxed::box_new([*bit])),
                                                    );
                                                }
                                            })
                                            .with_name("themeselect"),
                                    )
                                    .title("Select Theme")
                                    .dismiss_button("Cancel"),
                            );
                        },
                    ),
                ),
        );
    }
    pub fn ui() {
        let mut config = ASYNC.block_on(async { Config::new_or_default().await });
        let initial_config = config.clone();
        let mut siv = Cursive::new();
        let c_ = Ptr(&mut config);
        let prompt = config.binds.is_empty();
        siv.set_theme(Theme::retro());
        if let Some(mut home) = home_dir() {
            home.push(".ahqaiservertheme");
            if let Ok(x) = fs::read(&home) {
                let first_bit = &x[0];
                match *first_bit {
                    0 => {}
                    1 => siv.set_theme(Theme::terminal_default()),
                    _ => {}
                }
            }
        }
        siv.set_user_data(c_.clone());
        siv.set_global_callback('q', |x| x.quit());
        let mut tabs = TabPanel::new();
        let mut gene = LinearLayout::vertical();
        general(&mut gene, c_.clone());
        tabs.add_tab(
            ScrollView::new(gene).show_scrollbars(true).with_name("䷸ General"),
        );
        tabs.add_tab(llama::llama_page(c_.clone()));
        tabs.add_tab(auth::auth_page(&mut siv));
        tabs.add_tab(dbconf::db_page(c_.clone()));
        tabs.add_tab(
            ScrollView::new(
                    LinearLayout::vertical()
                        .child(
                            Button::new_raw(
                                "🖴 Save Changes and Exit",
                                |x| {
                                    x.quit();
                                },
                            ),
                        )
                        .child(
                            Button::new_raw(
                                "🖪 Backup current Config",
                                move |x| {
                                    let con: &mut Ptr<Config> = x.user_data().unwrap();
                                    let con = unsafe { &*con.0 };
                                    let file = ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "./config.bak.{0}.json",
                                                SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_secs(),
                                            ),
                                        )
                                    });
                                    fs::write(&file, to_string_pretty(con).unwrap()).unwrap();
                                    x.add_layer(
                                        Dialog::new()
                                            .title("Successful")
                                            .content(
                                                TextView::new(
                                                    ::alloc::__export::must_use({
                                                        ::alloc::fmt::format(
                                                            format_args!(
                                                                "Successfully backed up initial config at {0}",
                                                                file,
                                                            ),
                                                        )
                                                    }),
                                                ),
                                            )
                                            .dismiss_button("Ok"),
                                    );
                                },
                            ),
                        )
                        .child(
                            Button::new_raw(
                                "🖪 Backup Initial Config",
                                move |x| {
                                    let file = ::alloc::__export::must_use({
                                        ::alloc::fmt::format(
                                            format_args!(
                                                "./config.bak.{0}.json",
                                                SystemTime::now()
                                                    .duration_since(UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_secs(),
                                            ),
                                        )
                                    });
                                    fs::write(&file, to_string_pretty(&initial_config).unwrap())
                                        .unwrap();
                                    x.add_layer(
                                        Dialog::new()
                                            .title("Successful")
                                            .content(
                                                TextView::new(
                                                    ::alloc::__export::must_use({
                                                        ::alloc::fmt::format(
                                                            format_args!(
                                                                "Successfully backed up initial config at {0}",
                                                                file,
                                                            ),
                                                        )
                                                    }),
                                                ),
                                            )
                                            .dismiss_button("Ok"),
                                    );
                                },
                            ),
                        ),
                )
                .show_scrollbars(true)
                .with_name("🖫 Save"),
        );
        _ = tabs.set_active_tab("䷸ General");
        siv.add_layer(
            Dialog::around(tabs.with_name("tabs"))
                .title("AHQ-AI Server Configuration Utility")
                .full_screen(),
        );
        if prompt {
            siv.add_layer(
                Dialog::around(
                        TextView::new(
                            "Please set up hostnames and ports under `☸ General`!",
                        ),
                    )
                    .title("Important")
                    .dismiss_button("Ok"),
            );
        }
        if let None = &config.admin_pass_hash {
            siv.add_layer(
                Dialog::around(
                        LinearLayout::vertical()
                            .child(TextView::new("Set a server administrator password"))
                            .child(EditView::new().secret().with_name("pass")),
                    )
                    .button(
                        "Set",
                        |x| {
                            let pass = x
                                .call_on_name("pass", |x: &mut EditView| x.get_content())
                                .unwrap();
                            if pass.len() > 8 {
                                let c_: &mut Ptr<Config> = x.user_data().unwrap();
                                c_.admin_pass_hash = Some(
                                    hash_server_pass(pass.as_str()).unwrap(),
                                );
                                x.pop_layer();
                            } else {
                                x.add_layer(
                                    Dialog::around(
                                            TextView::new("Must be more than 8 characters"),
                                        )
                                        .dismiss_button("Ok"),
                                );
                            }
                        },
                    ),
            );
        }
        siv.run();
        ASYNC
            .block_on(async move {
                config.save_config().await.expect("Unable to save edited config");
            });
    }
    pub struct Ptr<T>(*mut T);
    #[automatically_derived]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for Ptr<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Ptr", &&self.0)
        }
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone> ::core::clone::Clone for Ptr<T> {
        #[inline]
        fn clone(&self) -> Ptr<T> {
            Ptr(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl<T: ::core::marker::Copy> ::core::marker::Copy for Ptr<T> {}
    unsafe impl<T> Send for Ptr<T> {}
    unsafe impl<T> Sync for Ptr<T> {}
    impl<T> Deref for Ptr<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            unsafe { &*self.0 }
        }
    }
    impl<T> DerefMut for Ptr<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.0 }
        }
    }
    fn binds(s: Ptr<Config>) -> LinearLayout {
        LinearLayout::horizontal()
            .child(TextView::new("🖳 Hostnames and Ports").full_width())
            .child(
                Button::new_raw(
                        "View ↗",
                        move |x| {
                            x.add_layer(bind::bind(s.clone()));
                        },
                    )
                    .with_name("host"),
            )
    }
}
pub mod auth {
    use crate::{
        auth::{
            authserver::{
                AuthServer, moka::MokaTestingDB, mongodb::MongodbClient, tikv::TikvClient,
            },
            cache::{AsyncCaching, moka::MokaSessions, redis::RedisSessions},
            hash::HashingAgent,
        },
        server::CONFIG,
        structs::{
            Authentication, db::{AuthDbConfig, CacheConfig},
            error::{Returns, ServerError},
        },
    };
    use base64::{Engine as _, engine::general_purpose};
    use log::warn;
    use rand::{Rng, seq::IndexedRandom};
    use std::{sync::LazyLock, time::{SystemTime, UNIX_EPOCH}};
    use tokio::task::spawn_blocking;
    pub mod argon {
        use std::sync::LazyLock;
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
        use argon2::{
            Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
            RECOMMENDED_SALT_LEN, Version, password_hash::SaltString,
        };
        use base64::{Engine, prelude::BASE64_STANDARD};
        use rand::{TryRngCore, rngs::OsRng};
        use zeroize::Zeroize;
        use crate::{
            server::CONFIG,
            structs::{
                Authentication, Config, db::{AuthDbConfig, CacheConfig},
                error::{Returns, ServerError},
            },
        };
        const KEY_LEN: usize = 32;
        static KEYARGON: LazyLock<Argon2> = LazyLock::new(|| {
            let iterations = 2;
            let memory = 32;
            let params = Params::new(memory * 1024, iterations, 1, Some(KEY_LEN))
                .unwrap();
            Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        });
        static HASHARGON: LazyLock<Argon2> = LazyLock::new(|| {
            let Authentication::Account { max_memory, time_cost, .. } = CONFIG
                .authentication
                .clone() else {
                ::core::panicking::panic("internal error: entered unreachable code")
            };
            let params = Params::new(max_memory * 1024, time_cost, 1, None).unwrap();
            Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        });
        pub static SALT_LEN: usize = RECOMMENDED_SALT_LEN * 2;
        pub fn hash_pass(pwd: &str, rng: &mut OsRng) -> Returns<String> {
            let mut salt_bytes = [0u8; SALT_LEN];
            rng.try_fill_bytes(&mut salt_bytes)?;
            let data: String = HASHARGON
                .hash_password(
                    pwd.as_bytes(),
                    &SaltString::encode_b64(&salt_bytes)
                        .map_err(|x| ServerError::ArgonErr(x))?,
                )
                .map_err(|x| ServerError::ArgonErr(x))?
                .to_string();
            Ok(data)
        }
        pub fn verify(pwd: &str, hash: &str) -> Returns<bool> {
            Ok(
                PasswordHash::new(hash)
                    .map_err(|x| ServerError::ArgonErr(x))?
                    .verify_password(&[&*HASHARGON], pwd)
                    .ok()
                    .is_some(),
            )
        }
        pub mod server {
            use crate::auth::argon::SALT_LEN;
            use crate::structs::error::ServerError;
            use crate::{auth::argon::KEYARGON, structs::error::Returns};
            use argon2::{PasswordHash, PasswordHasher, password_hash::SaltString};
            use rand::{TryRngCore, rngs::OsRng};
            /// Use only in the Terminal User Interface
            pub fn hash_server_pass(pwd: &str) -> Returns<String> {
                let mut salt_bytes = [0u8; SALT_LEN];
                OsRng::default().try_fill_bytes(&mut salt_bytes)?;
                let data: String = KEYARGON
                    .hash_password(
                        pwd.as_bytes(),
                        &SaltString::encode_b64(&salt_bytes)
                            .map_err(|x| ServerError::ArgonErr(x))?,
                    )
                    .map_err(|x| ServerError::ArgonErr(x))?
                    .to_string();
                Ok(data)
            }
            /// Use only in the Terminal User Interface
            pub fn verify_server_pass(pwd: &str, hash: &str) -> Returns<bool> {
                Ok(
                    PasswordHash::new(hash)
                        .map_err(|x| ServerError::ArgonErr(x))?
                        .verify_password(&[&*KEYARGON], pwd)
                        .ok()
                        .is_some(),
                )
            }
        }
        const NONCE_LEN: usize = 12;
        pub fn encrypt_with_key(pwd: &str, data: &str) -> String {
            let salt = {
                let mut salt_bytes = [0u8; SALT_LEN];
                OsRng::default().try_fill_bytes(&mut salt_bytes).unwrap();
                salt_bytes
            };
            let mut key = {
                let mut key_bytes = [0u8; KEY_LEN];
                KEYARGON
                    .hash_password_into(pwd.as_bytes(), salt.as_slice(), &mut key_bytes)
                    .unwrap();
                key_bytes
            };
            let aes = { Aes256Gcm::new_from_slice(&key).unwrap() };
            let nonce_slice = {
                let mut nonce_slice = [0u8; NONCE_LEN];
                OsRng::default().try_fill_bytes(&mut nonce_slice).unwrap();
                nonce_slice
            };
            let nonce = Nonce::from_slice(&nonce_slice);
            let ciphertext_with_tag = aes.encrypt(nonce, data.as_bytes()).unwrap();
            key.zeroize();
            let mut out = Vec::from(salt);
            out.extend(nonce_slice);
            out.extend(ciphertext_with_tag.into_iter());
            BASE64_STANDARD.encode(out)
        }
        /// # WARNING
        /// This functions returns an empty string if the data provided
        /// is empty. Please be informed
        pub fn decrypt_with_key(pwd: &str, data: &str) -> String {
            if data.is_empty() {
                return String::new();
            }
            let raw = BASE64_STANDARD.decode(data).unwrap();
            let salt_slice = &raw[0..SALT_LEN];
            let nonce_slice = &raw[SALT_LEN..(SALT_LEN + NONCE_LEN)];
            let nonce = Nonce::from_slice(nonce_slice);
            let cipher = &raw[(SALT_LEN + NONCE_LEN)..];
            let mut key = {
                let mut key_bytes = [0u8; KEY_LEN];
                KEYARGON
                    .hash_password_into(pwd.as_bytes(), salt_slice, &mut key_bytes)
                    .unwrap();
                key_bytes
            };
            let aes = { Aes256Gcm::new_from_slice(&key).unwrap() };
            key.zeroize();
            let ciphertext_with_tag = aes.decrypt(nonce, cipher).unwrap();
            String::from_utf8(ciphertext_with_tag).unwrap()
        }
        pub fn migrate_key(old_pass: &str, new_pass: &str, encrypted: &str) -> String {
            let data = decrypt_with_key(old_pass, encrypted);
            encrypt_with_key(new_pass, &data)
        }
        pub fn migrate_config(old_pass: &str, new_pass: &str, config: &mut Config) {
            {
                config
                    .llama
                    .models
                    .iter_mut()
                    .for_each(|(_, v)| {
                        if let Some(api) = &mut v.apikey {
                            *api = migrate_key(old_pass, new_pass, &api)
                                .into_boxed_str();
                        }
                    });
            }
            {
                match &mut config.database.authdb {
                    AuthDbConfig::Mongodb { url } => {
                        *url = migrate_key(old_pass, new_pass, &url).into_boxed_str();
                    }
                    AuthDbConfig::Tikv { endpoints, tls_config, .. } => {
                        endpoints
                            .iter_mut()
                            .for_each(|data| {
                                *data = migrate_key(old_pass, new_pass, &data)
                                    .into_boxed_str();
                            });
                        if let Some(conf) = tls_config {
                            conf.ca_path = migrate_key(old_pass, new_pass, &conf.ca_path)
                                .into_boxed_str();
                            conf.cert_path = migrate_key(
                                    old_pass,
                                    new_pass,
                                    &conf.cert_path,
                                )
                                .into_boxed_str();
                            conf.key_path = migrate_key(
                                    old_pass,
                                    new_pass,
                                    &conf.key_path,
                                )
                                .into_boxed_str();
                        }
                    }
                    AuthDbConfig::Moka { .. } => {}
                }
            }
            {
                match &mut config.database.cache {
                    CacheConfig::Moka => {}
                    CacheConfig::Redis { url } => {
                        *url = migrate_key(old_pass, new_pass, &url).into_boxed_str();
                    }
                }
            }
        }
        pub fn decrypt_config(pass: &str, config: &mut Config) {
            {
                config
                    .llama
                    .models
                    .iter_mut()
                    .for_each(|(_, v)| {
                        if let Some(api) = &mut v.apikey {
                            *api = decrypt_with_key(pass, &api).into_boxed_str();
                        }
                    });
            }
            {
                match &mut config.database.authdb {
                    AuthDbConfig::Mongodb { url } => {
                        *url = decrypt_with_key(pass, &url).into_boxed_str();
                    }
                    AuthDbConfig::Tikv { endpoints, tls_config, .. } => {
                        endpoints
                            .iter_mut()
                            .for_each(|data| {
                                *data = decrypt_with_key(pass, &data).into_boxed_str();
                            });
                        if let Some(conf) = tls_config {
                            conf.ca_path = decrypt_with_key(pass, &conf.ca_path)
                                .into_boxed_str();
                            conf.cert_path = decrypt_with_key(pass, &conf.cert_path)
                                .into_boxed_str();
                            conf.key_path = decrypt_with_key(pass, &conf.key_path)
                                .into_boxed_str();
                        }
                    }
                    AuthDbConfig::Moka { .. } => {}
                }
            }
            {
                match &mut config.database.cache {
                    CacheConfig::Moka => {}
                    CacheConfig::Redis { url } => {
                        *url = decrypt_with_key(pass, &url).into_boxed_str();
                    }
                }
            }
        }
    }
    pub mod hash {
        use crossbeam_channel::{Sender, bounded};
        use ed25519_dalek::{Signature, SigningKey, ed25519::signature::SignerMut};
        use rand::rngs::OsRng;
        use std::thread;
        use std::thread::available_parallelism;
        use tokio::sync::oneshot::{Sender as OneshotSender, channel};
        use crate::auth::{INTEGRITY_KEY, argon};
        pub struct HashingAgent(Sender<HashResp>);
        pub enum HashResp {
            CheckHash { pass: String, hash: String, tx: OneshotSender<Option<bool>> },
            GenHash { pass: String, tx: OneshotSender<Option<String>> },
            Challenge { bytes: Vec<u8>, tx: OneshotSender<Option<Signature>> },
        }
        impl HashingAgent {
            pub fn new() -> Self {
                let threads = available_parallelism()
                    .expect("Unable to get parallelism")
                    .get() - 1;
                let (tx, rx) = bounded::<HashResp>(2 * threads);
                for _ in 0..threads {
                    let rxc = rx.clone();
                    thread::spawn(move || {
                        let mut signer = SigningKey::from_keypair_bytes(INTEGRITY_KEY)
                            .unwrap();
                        let mut rng = OsRng::default();
                        while let Ok(x) = rxc.recv() {
                            match x {
                                HashResp::GenHash { pass, tx } => {
                                    _ = tx.send(argon::hash_pass(&pass, &mut rng).ok());
                                }
                                HashResp::CheckHash { pass, hash, tx } => {
                                    _ = tx.send(argon::verify(&pass, &hash).ok());
                                }
                                HashResp::Challenge { bytes, tx } => {
                                    let sign = signer.try_sign(&bytes).ok();
                                    _ = tx.send(sign);
                                }
                            }
                        }
                    });
                }
                Self(tx)
            }
            /// # Cloning:
            /// This
            ///
            /// # Returns
            /// This function returns None in case of the server's queue being maxed out
            pub async fn verify_pass(&self, pass: &str, hash: &str) -> Option<bool> {
                if self.0.is_full() {
                    return None;
                }
                let hash: String = hash.to_owned();
                let pass: String = pass.to_owned();
                let (tx, rx) = channel::<Option<bool>>();
                self.0
                    .try_send(HashResp::CheckHash {
                        pass,
                        hash,
                        tx,
                    })
                    .ok()?;
                rx.await.ok()?
            }
            /// # Returns
            /// This function returns None in case of the server's queue being maxed out
            pub async fn gen_hash(&self, pass: &str) -> Option<String> {
                if self.0.is_full() {
                    return None;
                }
                let pass: String = pass.to_owned();
                let (tx, rx) = channel::<Option<String>>();
                self.0.try_send(HashResp::GenHash { pass, tx }).ok()?;
                rx.await.ok()?
            }
            /// # Returns
            /// This function returns None in case of the server's queue being maxed out
            pub async fn gen_signature(&self, data: &[u8]) -> Option<Signature> {
                if self.0.is_full() {
                    return None;
                }
                let bytes = data.to_owned();
                let (tx, rx) = channel::<Option<Signature>>();
                self.0.try_send(HashResp::Challenge { bytes, tx }).ok()?;
                rx.await.ok()?
            }
        }
        impl Default for HashingAgent {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub mod authserver {
        use crate::structs::error::Returns;
        use async_trait::async_trait;
        pub mod moka {
            use async_trait::async_trait;
            use moka::future::Cache;
            use crate::{
                auth::authserver::AuthServer, structs::error::{Returns, ServerError},
            };
            pub struct MokaTestingDB {
                cache: Cache<String, String>,
            }
            impl MokaTestingDB {
                pub fn new() -> Self {
                    Self {
                        cache: Cache::builder().build(),
                    }
                }
            }
            impl AuthServer for MokaTestingDB {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn get<'a, 'async_trait>(
                    &'a self,
                    uid: &'a str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Option<String>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Option<String>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<Option<String>> = {
                            Ok(__self.cache.get(uid).await)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn search<'a, 'async_trait>(
                    &'a self,
                    __arg1: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Vec<Vec<u8>>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Vec<Vec<u8>>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __arg1 = __arg1;
                        let __ret: Returns<Vec<Vec<u8>>> = {
                            Err(ServerError::StringConvertErr)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn exists<'a, 'async_trait>(
                    &'a self,
                    uid: &'a str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<bool>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<bool>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<bool> = {
                            Ok(__self.cache.contains_key(uid))
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn update<'a, 'async_trait>(
                    &'a self,
                    uid: String,
                    data: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let uid = uid;
                        let data = data;
                        let __ret: Returns<()> = {
                            __self.cache.insert(uid, data).await;
                            Ok(())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn remove<'a, 'async_trait>(
                    &'a self,
                    uid: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let uid = uid;
                        let __ret: Returns<()> = {
                            __self.cache.remove(&uid).await;
                            Ok(())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
        }
        pub mod mongodb {
            use async_trait::async_trait;
            use futures::StreamExt;
            use mongodb::{Client, Collection, bson::doc};
            use serde::{Deserialize, Serialize};
            use tokio::spawn;
            use crate::{
                auth::authserver::AuthServer, server::DECRYPTED_CONFIG,
                structs::{db::AuthDbConfig, error::Returns},
            };
            pub struct UserOrAuthToken {
                #[serde(rename = "_id")]
                pub id: String,
                pub hash: String,
            }
            #[doc(hidden)]
            #[allow(
                non_upper_case_globals,
                unused_attributes,
                unused_qualifications,
                clippy::absolute_paths,
            )]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl _serde::Serialize for UserOrAuthToken {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                    where
                        __S: _serde::Serializer,
                    {
                        let mut __serde_state = _serde::Serializer::serialize_struct(
                            __serializer,
                            "UserOrAuthToken",
                            false as usize + 1 + 1,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "_id",
                            &self.id,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "hash",
                            &self.hash,
                        )?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
            #[doc(hidden)]
            #[allow(
                non_upper_case_globals,
                unused_attributes,
                unused_qualifications,
                clippy::absolute_paths,
            )]
            const _: () = {
                #[allow(unused_extern_crates, clippy::useless_attribute)]
                extern crate serde as _serde;
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for UserOrAuthToken {
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private228::Formatter,
                            ) -> _serde::__private228::fmt::Result {
                                _serde::__private228::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private228::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private228::Ok(__Field::__field0),
                                    1u64 => _serde::__private228::Ok(__Field::__field1),
                                    _ => _serde::__private228::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private228::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                            {
                                match __value {
                                    "_id" => _serde::__private228::Ok(__Field::__field0),
                                    "hash" => _serde::__private228::Ok(__Field::__field1),
                                    _ => _serde::__private228::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private228::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                            {
                                match __value {
                                    b"_id" => _serde::__private228::Ok(__Field::__field0),
                                    b"hash" => _serde::__private228::Ok(__Field::__field1),
                                    _ => _serde::__private228::Ok(__Field::__ignore),
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private228::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private228::PhantomData<UserOrAuthToken>,
                            lifetime: _serde::__private228::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = UserOrAuthToken;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private228::Formatter,
                            ) -> _serde::__private228::fmt::Result {
                                _serde::__private228::Formatter::write_str(
                                    __formatter,
                                    "struct UserOrAuthToken",
                                )
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private228::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 = match _serde::de::SeqAccess::next_element::<
                                    String,
                                >(&mut __seq)? {
                                    _serde::__private228::Some(__value) => __value,
                                    _serde::__private228::None => {
                                        return _serde::__private228::Err(
                                            _serde::de::Error::invalid_length(
                                                0usize,
                                                &"struct UserOrAuthToken with 2 elements",
                                            ),
                                        );
                                    }
                                };
                                let __field1 = match _serde::de::SeqAccess::next_element::<
                                    String,
                                >(&mut __seq)? {
                                    _serde::__private228::Some(__value) => __value,
                                    _serde::__private228::None => {
                                        return _serde::__private228::Err(
                                            _serde::de::Error::invalid_length(
                                                1usize,
                                                &"struct UserOrAuthToken with 2 elements",
                                            ),
                                        );
                                    }
                                };
                                _serde::__private228::Ok(UserOrAuthToken {
                                    id: __field0,
                                    hash: __field1,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private228::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private228::Option<String> = _serde::__private228::None;
                                let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                                while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                    __Field,
                                >(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private228::Option::is_some(&__field0) {
                                                return _serde::__private228::Err(
                                                    <__A::Error as _serde::de::Error>::duplicate_field("_id"),
                                                );
                                            }
                                            __field0 = _serde::__private228::Some(
                                                _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private228::Option::is_some(&__field1) {
                                                return _serde::__private228::Err(
                                                    <__A::Error as _serde::de::Error>::duplicate_field("hash"),
                                                );
                                            }
                                            __field1 = _serde::__private228::Some(
                                                _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                            );
                                        }
                                        _ => {
                                            let _ = _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private228::Some(__field0) => __field0,
                                    _serde::__private228::None => {
                                        _serde::__private228::de::missing_field("_id")?
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private228::Some(__field1) => __field1,
                                    _serde::__private228::None => {
                                        _serde::__private228::de::missing_field("hash")?
                                    }
                                };
                                _serde::__private228::Ok(UserOrAuthToken {
                                    id: __field0,
                                    hash: __field1,
                                })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] = &["_id", "hash"];
                        _serde::Deserializer::deserialize_struct(
                            __deserializer,
                            "UserOrAuthToken",
                            FIELDS,
                            __Visitor {
                                marker: _serde::__private228::PhantomData::<
                                    UserOrAuthToken,
                                >,
                                lifetime: _serde::__private228::PhantomData,
                            },
                        )
                    }
                }
            };
            #[automatically_derived]
            impl ::core::fmt::Debug for UserOrAuthToken {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "UserOrAuthToken",
                        "id",
                        &self.id,
                        "hash",
                        &&self.hash,
                    )
                }
            }
            pub struct MongodbClient {
                _client: Client,
                auth_hash: Collection<UserOrAuthToken>,
            }
            impl MongodbClient {
                pub async fn new() -> Self {
                    let AuthDbConfig::Mongodb { url } = &DECRYPTED_CONFIG
                        .read()
                        .await
                        .database
                        .authdb else {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    };
                    let client = Client::with_uri_str(url)
                        .await
                        .expect("Failed to connect to MongoDB");
                    let auth_hash = client
                        .database("auth")
                        .collection::<UserOrAuthToken>("auth");
                    Self { _client: client, auth_hash }
                }
            }
            impl AuthServer for MongodbClient {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn get<'a, 'async_trait>(
                    &'a self,
                    uid: &'a str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Option<String>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Option<String>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<Option<String>> = {
                            let auth_hash = __self.auth_hash.clone();
                            let doc = {
                                let mut object = ::bson::Document::new();
                                object
                                    .insert::<
                                        _,
                                        ::bson::Bson,
                                    >(
                                        ("_id"),
                                        <_ as ::std::convert::Into<::bson::Bson>>::into(uid),
                                    );
                                object
                            };
                            tokio::spawn(async move {
                                    let out = auth_hash.find_one(doc).await?;
                                    Ok(out.map(|u| u.hash))
                                })
                                .await?
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn search<'a, 'async_trait>(
                    &'a self,
                    prefix: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Vec<Vec<u8>>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Vec<Vec<u8>>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let prefix = prefix;
                        let __ret: Returns<Vec<Vec<u8>>> = {
                            let auth_hash = __self.auth_hash.clone();
                            tokio::spawn(async move {
                                    Ok(
                                        auth_hash
                                            .find({
                                                let mut object = ::bson::Document::new();
                                                object
                                                    .insert::<
                                                        _,
                                                        ::bson::Bson,
                                                    >(
                                                        ("_id"),
                                                        ::bson::Bson::Document({
                                                            let mut object = ::bson::Document::new();
                                                            object
                                                                .insert::<
                                                                    _,
                                                                    ::bson::Bson,
                                                                >(
                                                                    ("$regex"),
                                                                    <_ as ::std::convert::Into<
                                                                        ::bson::Bson,
                                                                    >>::into(
                                                                        ::alloc::__export::must_use({
                                                                            ::alloc::fmt::format(format_args!("^{0}.*", prefix))
                                                                        }),
                                                                    ),
                                                                );
                                                            object
                                                                .insert::<
                                                                    _,
                                                                    ::bson::Bson,
                                                                >(
                                                                    ("$options"),
                                                                    <_ as ::std::convert::Into<::bson::Bson>>::into("i"),
                                                                );
                                                            object
                                                        }),
                                                    );
                                                object
                                            })
                                            .limit(100)
                                            .await?
                                            .filter(|x| {
                                                let out = x.is_ok();
                                                async move { out }
                                            })
                                            .map(|x| x.unwrap().id.into_bytes())
                                            .collect::<Vec<_>>()
                                            .await,
                                    )
                                })
                                .await?
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn exists<'a, 'async_trait>(
                    &'a self,
                    uid: &'a str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<bool>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<bool>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<bool> = {
                            let auth_hash = __self.auth_hash.clone();
                            let doc = {
                                let mut object = ::bson::Document::new();
                                object
                                    .insert::<
                                        _,
                                        ::bson::Bson,
                                    >(
                                        ("_id"),
                                        <_ as ::std::convert::Into<::bson::Bson>>::into(uid),
                                    );
                                object
                            };
                            spawn(async move {
                                    Ok(auth_hash.find_one(doc).await?.is_some())
                                })
                                .await?
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn update<'a, 'async_trait>(
                    &'a self,
                    uid: String,
                    data: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let uid = uid;
                        let data = data;
                        let __ret: Returns<()> = {
                            let auth_hash = __self.auth_hash.clone();
                            spawn(async move {
                                    auth_hash
                                        .find_one_and_update(
                                            {
                                                let mut object = ::bson::Document::new();
                                                object
                                                    .insert::<
                                                        _,
                                                        ::bson::Bson,
                                                    >(
                                                        ("_id"),
                                                        <_ as ::std::convert::Into<::bson::Bson>>::into(&uid),
                                                    );
                                                object
                                            },
                                            {
                                                let mut object = ::bson::Document::new();
                                                object
                                                    .insert::<
                                                        _,
                                                        ::bson::Bson,
                                                    >(
                                                        ("$set"),
                                                        ::bson::Bson::Document({
                                                            let mut object = ::bson::Document::new();
                                                            object
                                                                .insert::<
                                                                    _,
                                                                    ::bson::Bson,
                                                                >(
                                                                    ("hash"),
                                                                    <_ as ::std::convert::Into<::bson::Bson>>::into(data),
                                                                );
                                                            object
                                                        }),
                                                    );
                                                object
                                            },
                                        )
                                        .upsert(true)
                                        .await?;
                                    Ok(())
                                })
                                .await?
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn remove<'a, 'async_trait>(
                    &'a self,
                    uid: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let uid = uid;
                        let __ret: Returns<()> = {
                            let auth_hash = __self.auth_hash.clone();
                            spawn(async move {
                                    auth_hash
                                        .delete_one({
                                            let mut object = ::bson::Document::new();
                                            object
                                                .insert::<
                                                    _,
                                                    ::bson::Bson,
                                                >(
                                                    ("_id"),
                                                    <_ as ::std::convert::Into<::bson::Bson>>::into(uid),
                                                );
                                            object
                                        })
                                        .await?;
                                    Ok(())
                                })
                                .await?
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
        }
        pub mod tikv {
            use async_trait::async_trait;
            use log::*;
            use std::{path::PathBuf, str::FromStr, time::Duration};
            use tikv_client::{
                BoundRange, Config, Error, Key, RawClient, TransactionClient,
            };
            use tokio::time::sleep;
            use crate::{
                auth::authserver::AuthServer, server::DECRYPTED_CONFIG,
                structs::{db::AuthDbConfig, error::{Returns, ServerError}},
            };
            pub struct TikvClient {
                raw: RawClient,
                transactional: TransactionClient,
            }
            impl TikvClient {
                pub async fn new() -> Self {
                    {
                        {
                            let lvl = ::log::Level::Info;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!("Connecting to TiKV Database"),
                                    lvl,
                                    &(
                                        "ahqai_server::auth::authserver::tikv",
                                        "ahqai_server::auth::authserver::tikv",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    let mut config = Config::default();
                    let AuthDbConfig::Tikv { endpoints, tls_config, timeout_secs } = &DECRYPTED_CONFIG
                        .read()
                        .await
                        .database
                        .authdb else {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!("This is not TiKV"),
                            );
                        };
                    };
                    if *timeout_secs > 0 {
                        config.timeout = Duration::from_secs(*timeout_secs as u64);
                    }
                    if let Some(tls) = tls_config {
                        config.ca_path = Some(
                            PathBuf::from_str(&tls.ca_path).expect("Invalid `ca_path`"),
                        );
                        config.cert_path = Some(
                            PathBuf::from_str(&tls.cert_path)
                                .expect("Invalid `cert_path`"),
                        );
                        config.key_path = Some(
                            PathBuf::from_str(&tls.key_path).expect("Invalid `key_path`"),
                        );
                    }
                    let endpoints = endpoints
                        .iter()
                        .map(|x| x as &str)
                        .collect::<Vec<_>>();
                    Self {
                        raw: RawClient::new_with_config(
                                endpoints.clone(),
                                config.clone(),
                            )
                            .await
                            .expect("Unable to initialize Database connection"),
                        transactional: TransactionClient::new_with_config(
                                endpoints,
                                config,
                            )
                            .await
                            .expect("Unable to initialize Database connection"),
                    }
                }
            }
            impl AuthServer for TikvClient {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn get<'a, 'async_trait>(
                    &'a self,
                    uid: &'a str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Option<String>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Option<String>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<Option<String>> = {
                            let out = __self.raw.get(uid.to_owned()).await?;
                            Ok(out.map(|x| String::from_utf8(x).ok()).flatten())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn search<'a, 'async_trait>(
                    &'a self,
                    prefix: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Vec<Vec<u8>>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Vec<Vec<u8>>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let prefix = prefix;
                        let __ret: Returns<Vec<Vec<u8>>> = {
                            let start = prefix.into_bytes();
                            let mut end = start.clone();
                            end.push(255);
                            let range = BoundRange::from(
                                Key::from(start)..Key::from(end),
                            );
                            Ok(
                                __self
                                    .raw
                                    .scan_keys(range, 100)
                                    .await?
                                    .into_iter()
                                    .map(Vec::from)
                                    .collect::<Vec<_>>(),
                            )
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn exists<'a, 'async_trait>(
                    &'a self,
                    uid: &'a str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<bool>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<bool>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<bool> = {
                            let out = __self.raw.get(uid.to_owned()).await?;
                            Ok(out.is_some())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn update<'a, 'async_trait>(
                    &'a self,
                    uid: String,
                    data: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let uid = uid;
                        let data = data;
                        let __ret: Returns<()> = {
                            for attempt in 1..=5 {
                                let mut txn = __self
                                    .transactional
                                    .begin_optimistic()
                                    .await?;
                                txn.put(uid.clone(), data.clone()).await?;
                                match txn.commit().await {
                                    Ok(_) => return Ok(()),
                                    Err(e) => {
                                        if should_retry(&e) {
                                            sleep(
                                                    Duration::from_millis(
                                                        (30 * 2u64.pow(attempt - 1)).min(1000),
                                                    ),
                                                )
                                                .await;
                                            continue;
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(ServerError::RetryFailed)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn remove<'a, 'async_trait>(
                    &'a self,
                    uid: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'a: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let uid = uid;
                        let __ret: Returns<()> = {
                            for attempt in 1..=5 {
                                let mut txn = __self
                                    .transactional
                                    .begin_optimistic()
                                    .await?;
                                txn.delete(uid.clone()).await?;
                                match txn.commit().await {
                                    Ok(_) => return Ok(()),
                                    Err(e) => {
                                        if should_retry(&e) {
                                            sleep(
                                                    Duration::from_millis(
                                                        (30 * 2u64.pow(attempt - 1)).min(1000),
                                                    ),
                                                )
                                                .await;
                                            continue;
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(ServerError::RetryFailed)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
            fn should_retry(e: &Error) -> bool {
                #[allow(non_exhaustive_omitted_patterns)]
                match e {
                    Error::KeyError(_)
                    | Error::PessimisticLockError { .. }
                    | Error::RegionError(_)
                    | Error::LeaderNotFound { .. }
                    | Error::UndeterminedError(_) => true,
                    _ => false,
                }
            }
        }
        pub(crate) trait AuthServer {
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn get<'a, 'async_trait>(
                &'a self,
                uid: &'a str,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Returns<Option<String>>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'a: 'async_trait,
                Self: 'async_trait;
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn search<'a, 'async_trait>(
                &'a self,
                prefix: String,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Returns<Vec<Vec<u8>>>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'a: 'async_trait,
                Self: 'async_trait;
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn exists<'a, 'async_trait>(
                &'a self,
                uid: &'a str,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Returns<bool>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'a: 'async_trait,
                Self: 'async_trait;
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn update<'a, 'async_trait>(
                &'a self,
                uid: String,
                data: String,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Returns<()>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'a: 'async_trait,
                Self: 'async_trait;
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn remove<'a, 'async_trait>(
                &'a self,
                uid: String,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Returns<()>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'a: 'async_trait,
                Self: 'async_trait;
        }
    }
    pub mod cache {
        use async_trait::async_trait;
        use crate::structs::error::Returns;
        pub mod moka {
            use std::time::Duration;
            use async_trait::async_trait;
            use moka::future::Cache;
            use crate::{auth::cache::AsyncCaching, structs::error::Returns};
            pub struct MokaSessions {
                cache: Cache<String, String>,
            }
            impl MokaSessions {
                pub fn new() -> Self {
                    Self {
                        cache: Cache::builder()
                            .time_to_live(Duration::from_days(30))
                            .build(),
                    }
                }
            }
            impl AsyncCaching for MokaSessions {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn get<'life0, 'life1, 'async_trait>(
                    &'life0 self,
                    key: &'life1 str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Option<String>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Option<String>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<Option<String>> = {
                            Ok(__self.cache.get(key).await)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn insert<'life0, 'life1, 'async_trait>(
                    &'life0 self,
                    key: &'life1 str,
                    value: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let value = value;
                        let __ret: Returns<()> = {
                            __self.cache.insert(key.to_owned(), value).await;
                            Ok(())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
        }
        pub mod redis {
            use async_trait::async_trait;
            use redis::{AsyncTypedCommands, Client, aio::MultiplexedConnection};
            use crate::{
                auth::cache::AsyncCaching, server::DECRYPTED_CONFIG,
                structs::{db::CacheConfig, error::Returns},
            };
            pub struct RedisSessions {
                _redis: Client,
                conn: MultiplexedConnection,
            }
            impl RedisSessions {
                pub async fn new() -> Self {
                    let CacheConfig::Redis { url } = &DECRYPTED_CONFIG
                        .read()
                        .await
                        .database
                        .cache else {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    };
                    let redis = Client::open(url as &str)
                        .expect("Failed to initialize redis connection");
                    let conn = redis
                        .get_multiplexed_async_connection()
                        .await
                        .expect("Unable to get REDIS Connection");
                    RedisSessions {
                        _redis: redis,
                        conn,
                    }
                }
            }
            const THIRTY_DAYS_IN_SECS: u64 = 30 * 24 * 60 * 60;
            impl AsyncCaching for RedisSessions {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn get<'life0, 'life1, 'async_trait>(
                    &'life0 self,
                    key: &'life1 str,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<Option<String>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<Option<String>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let __ret: Returns<Option<String>> = {
                            Ok(__self.conn.clone().get(key).await?)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn insert<'life0, 'life1, 'async_trait>(
                    &'life0 self,
                    key: &'life1 str,
                    value: String,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Returns<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Returns<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let __self = self;
                        let value = value;
                        let __ret: Returns<()> = {
                            __self
                                .conn
                                .clone()
                                .set_ex(key, value, THIRTY_DAYS_IN_SECS)
                                .await?;
                            Ok(())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
            }
        }
        pub trait AsyncCaching {
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn get<'life0, 'life1, 'async_trait>(
                &'life0 self,
                key: &'life1 str,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Returns<Option<String>>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                Self: 'async_trait;
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn insert<'life0, 'life1, 'async_trait>(
                &'life0 self,
                key: &'life1 str,
                value: String,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Returns<()>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                Self: 'async_trait;
        }
    }
    pub static INTEGRITY_KEY: &'static [u8; 64] = b"|\xa0lT\xe0\x8d\x08\xc1\x8f\xa8\xdb\xb2\x8b]\xd4x*$#\xd3\x8f\xae\xcdC\x88C\x8c\xee\x93\x11\xc3\x87Pg\x0b\x01\x8a^\xc1\xfa$\x7f\x1b\xbdt\x93\x0c\xf9s%\x81(\xee\xf6\xb2\xf7\xb4\xe2sZ-\xc40\xa7";
    pub static AGENT: LazyLock<HashingAgent> = LazyLock::new(|| HashingAgent::new());
    const TOKEN_ID_LENGTH: usize = 12;
    #[allow(dead_code)]
    pub(crate) struct AuthSessionManager {
        sessions: Box<dyn AsyncCaching + Send + Sync>,
        pub accounts: Box<dyn AuthServer + Send + Sync>,
        agent: &'static HashingAgent,
    }
    pub enum AccountCreateOutcome {
        UsernameExists,
        WeakPassword,
        InternalServerError,
        Successful,
        SuccessfulOut(String),
    }
    pub enum AccountCheckOutcome {
        NotFound,
        TooManyRequests,
        InvalidPassword,
        Some(String),
    }
    pub type AccountOrToken = (Box<str>, Box<str>);
    impl AuthSessionManager {
        pub async fn create() -> Self {
            let accounts: Box<dyn AuthServer + Send + Sync>;
            let sessions: Box<dyn AsyncCaching + Send + Sync>;
            match &CONFIG.database.authdb {
                AuthDbConfig::Moka {} => {
                    {
                        {
                            let lvl = ::log::Level::Warn;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!(
                                        "CRITICAL WARNING! YOU ARE USING MOKA DB WHICH NEITHER HAS PERSISTENCE NOR IS RECOMMENDED FOR PRODUCTION IN ANY MEANS. PLEASE SHIFT TO A MORE ROBUST DB IMPLEMENTATION LIKE MONGODB OR TIKV FOR EVEN A HOBBY SERVER.",
                                    ),
                                    lvl,
                                    &(
                                        "ahqai_server::auth",
                                        "ahqai_server::auth",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    accounts = Box::new(MokaTestingDB::new());
                }
                AuthDbConfig::Mongodb { .. } => {
                    accounts = Box::new(MongodbClient::new().await);
                }
                AuthDbConfig::Tikv { .. } => accounts = Box::new(TikvClient::new().await),
            };
            match &CONFIG.database.cache {
                CacheConfig::Moka => sessions = Box::new(MokaSessions::new()),
                CacheConfig::Redis { .. } => {
                    sessions = Box::new(RedisSessions::new().await);
                }
            };
            Self {
                sessions,
                accounts,
                agent: &*AGENT,
            }
        }
    }
    impl AuthSessionManager {
        pub async fn can_register(&self) -> bool {
            let Authentication::Account { registration_allowed, .. } = CONFIG
                .authentication else {
                return false;
            };
            registration_allowed
        }
        /// THIS ENDPOINT HAS ABSOLUTELY NO PROTECTION
        /// DEVS SHOULD USE `AuthSessionManager::can_register` first
        pub async fn register(
            &self,
            user: &str,
            pass: &str,
        ) -> Returns<AccountCreateOutcome> {
            if self.accounts.exists(user).await? {
                return Ok(AccountCreateOutcome::UsernameExists);
            }
            if !is_strong_password(pass).await {
                return Ok(AccountCreateOutcome::WeakPassword);
            }
            let Some(pwd_hash) = self.agent.gen_hash(pass).await else {
                return Ok(AccountCreateOutcome::InternalServerError);
            };
            self.accounts.update(user.to_owned(), pwd_hash).await?;
            Ok(AccountCreateOutcome::Successful)
        }
        pub async fn add_token(&self) -> Returns<AccountCreateOutcome> {
            let (key, (user, hash)) = gen_auth_token(self.agent).await?;
            self.accounts.update(user, hash).await?;
            Ok(AccountCreateOutcome::SuccessfulOut(key))
        }
        pub async fn is_valid_token(&self, token: &str) -> Returns<AccountCheckOutcome> {
            let Some((tok_id, token)) = token.split_once(".") else {
                return Ok(AccountCheckOutcome::NotFound);
            };
            self.is_valid_account(tok_id, token).await
        }
        pub async fn is_valid_account(
            &self,
            userid: &str,
            pass: &str,
        ) -> Returns<AccountCheckOutcome> {
            let Some(hash) = self.accounts.get(userid).await? else {
                return Ok(AccountCheckOutcome::NotFound);
            };
            let Some(x) = self.agent.verify_pass(pass, &hash).await else {
                return Ok(AccountCheckOutcome::TooManyRequests);
            };
            if !x {
                return Ok(AccountCheckOutcome::InvalidPassword);
            }
            if let Some(session) = self.sessions.get(userid).await? {
                let sess_cloned = ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("{0}.{1}", userid, session))
                });
                return Ok(AccountCheckOutcome::Some(sess_cloned));
            }
            let sess = gen_session_token()?;
            let sess_cloned = ::alloc::__export::must_use({
                ::alloc::fmt::format(format_args!("{0}.{1}", userid, sess))
            });
            self.sessions.insert(userid, sess).await?;
            Ok(AccountCheckOutcome::Some(sess_cloned))
        }
        pub async fn verify_session(&self, token: &str) -> bool {
            let Some((userid, session)) = token.split_once(".") else {
                return false;
            };
            self.sessions
                .get(userid)
                .await
                .ok()
                .flatten()
                .map(|x| session == (&x as &str))
                .is_some_and(|x| x)
        }
    }
    pub fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
    pub async fn is_strong_password(password: &str) -> bool {
        if password.len() < 8 {
            return false;
        }
        let mut uppercase = false;
        let mut lowercase = false;
        let mut digit = false;
        let mut special = false;
        password
            .chars()
            .any(|x| {
                if x.is_ascii_digit() {
                    digit = true;
                }
                if x.is_ascii_uppercase() {
                    uppercase = true;
                }
                if x.is_ascii_lowercase() {
                    lowercase = true;
                }
                if !x.is_ascii_alphanumeric() {
                    special = true;
                }
                digit && uppercase && lowercase && special
            })
    }
    pub const VALUES: [char; 64] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
        'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
        'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
        'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/',
    ];
    pub type Hashed = String;
    pub fn gen_uid() -> Returns<String> {
        let mut rng = rand::rng();
        let token = VALUES.choose_multiple(&mut rng, 32).collect::<String>();
        Ok(token)
    }
    pub async fn gen_auth_token(
        cpufarm: &HashingAgent,
    ) -> Returns<(String, (String, Hashed))> {
        let mut rng = rand::rng();
        let token = VALUES.choose_multiple(&mut rng, 128).collect::<String>();
        let mut token_key = String::from("tok:");
        token_key.extend(VALUES.choose_multiple(&mut rng, TOKEN_ID_LENGTH));
        let hashed = cpufarm
            .gen_hash(&token)
            .await
            .map_or_else(|| Err(ServerError::StringConvertErr), |x| Ok(x))?;
        let token_to_output = ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0}.{1}", token_key, token))
        });
        Ok((token_to_output, (token_key, hashed)))
    }
    pub async fn gen_session_token_async() -> Returns<String> {
        spawn_blocking(gen_session_token).await?
    }
    pub fn gen_session_token() -> Returns<String> {
        let mut rng = rand::rng();
        let mut token = ::alloc::vec::from_elem(0u8, 96);
        rng.fill(&mut token as &mut [u8]);
        let token = general_purpose::URL_SAFE_NO_PAD.encode(&token);
        Ok(token)
    }
}
pub(crate) mod structs {
    use std::collections::HashMap;
    use zeroize::{Zeroize, ZeroizeOnDrop};
    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string_pretty};
    use tokio::fs;
    use crate::structs::{db::DatabaseConfig, error::Returns};
    pub mod db {
        use zeroize::{Zeroize, ZeroizeOnDrop};
        use serde::{Deserialize, Serialize};
        pub struct DatabaseConfig {
            pub authdb: AuthDbConfig,
            pub cache: CacheConfig,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for DatabaseConfig {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "DatabaseConfig",
                    "authdb",
                    &self.authdb,
                    "cache",
                    &&self.cache,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for DatabaseConfig {
            #[inline]
            fn clone(&self) -> DatabaseConfig {
                DatabaseConfig {
                    authdb: ::core::clone::Clone::clone(&self.authdb),
                    cache: ::core::clone::Clone::clone(&self.cache),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for DatabaseConfig {
            #[inline]
            fn default() -> DatabaseConfig {
                DatabaseConfig {
                    authdb: ::core::default::Default::default(),
                    cache: ::core::default::Default::default(),
                }
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for DatabaseConfig {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "DatabaseConfig",
                        false as usize + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "authdb",
                        &self.authdb,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "cache",
                        &self.cache,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for DatabaseConfig {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __ignore,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                _ => _serde::__private228::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "authdb" => _serde::__private228::Ok(__Field::__field0),
                                "cache" => _serde::__private228::Ok(__Field::__field1),
                                _ => _serde::__private228::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"authdb" => _serde::__private228::Ok(__Field::__field0),
                                b"cache" => _serde::__private228::Ok(__Field::__field1),
                                _ => _serde::__private228::Ok(__Field::__ignore),
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private228::PhantomData<DatabaseConfig>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = DatabaseConfig;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct DatabaseConfig",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                AuthDbConfig,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct DatabaseConfig with 2 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                CacheConfig,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct DatabaseConfig with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(DatabaseConfig {
                                authdb: __field0,
                                cache: __field1,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<
                                AuthDbConfig,
                            > = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<
                                CacheConfig,
                            > = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("authdb"),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                AuthDbConfig,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("cache"),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                CacheConfig,
                                            >(&mut __map)?,
                                        );
                                    }
                                    _ => {
                                        let _ = _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(&mut __map)?;
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("authdb")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("cache")?
                                }
                            };
                            _serde::__private228::Ok(DatabaseConfig {
                                authdb: __field0,
                                cache: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["authdb", "cache"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "DatabaseConfig",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<DatabaseConfig>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        impl ::zeroize::Zeroize for DatabaseConfig {
            fn zeroize(&mut self) {
                match self {
                    #[allow(unused_variables)]
                    DatabaseConfig { authdb, cache } => {
                        authdb.zeroize();
                        cache.zeroize()
                    }
                    _ => {}
                }
            }
        }
        impl Drop for DatabaseConfig {
            fn drop(&mut self) {
                use ::zeroize::__internal::AssertZeroize;
                use ::zeroize::__internal::AssertZeroizeOnDrop;
                match self {
                    #[allow(unused_variables)]
                    DatabaseConfig { authdb, cache } => {
                        authdb.zeroize_or_on_drop();
                        cache.zeroize_or_on_drop()
                    }
                    _ => {}
                }
            }
        }
        #[doc(hidden)]
        impl ::zeroize::ZeroizeOnDrop for DatabaseConfig {}
        #[serde(tag = "db")]
        pub enum AuthDbConfig {
            #[serde(rename = "moka")]
            Moka {},
            #[serde(rename = "mongodb")]
            Mongodb { url: Box<str> },
            #[serde(rename = "tikv")]
            Tikv {
                endpoints: Box<[Box<str>]>,
                tls_config: Option<TlsConfig>,
                #[serde(default = "def_timeout")]
                timeout_secs: u64,
            },
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for AuthDbConfig {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    AuthDbConfig::Moka {} => ::core::fmt::Formatter::write_str(f, "Moka"),
                    AuthDbConfig::Mongodb { url: __self_0 } => {
                        ::core::fmt::Formatter::debug_struct_field1_finish(
                            f,
                            "Mongodb",
                            "url",
                            &__self_0,
                        )
                    }
                    AuthDbConfig::Tikv {
                        endpoints: __self_0,
                        tls_config: __self_1,
                        timeout_secs: __self_2,
                    } => {
                        ::core::fmt::Formatter::debug_struct_field3_finish(
                            f,
                            "Tikv",
                            "endpoints",
                            __self_0,
                            "tls_config",
                            __self_1,
                            "timeout_secs",
                            &__self_2,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for AuthDbConfig {
            #[inline]
            fn clone(&self) -> AuthDbConfig {
                match self {
                    AuthDbConfig::Moka {} => AuthDbConfig::Moka {},
                    AuthDbConfig::Mongodb { url: __self_0 } => {
                        AuthDbConfig::Mongodb {
                            url: ::core::clone::Clone::clone(__self_0),
                        }
                    }
                    AuthDbConfig::Tikv {
                        endpoints: __self_0,
                        tls_config: __self_1,
                        timeout_secs: __self_2,
                    } => {
                        AuthDbConfig::Tikv {
                            endpoints: ::core::clone::Clone::clone(__self_0),
                            tls_config: ::core::clone::Clone::clone(__self_1),
                            timeout_secs: ::core::clone::Clone::clone(__self_2),
                        }
                    }
                }
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for AuthDbConfig {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        AuthDbConfig::Moka {} => {
                            let mut __serde_state = _serde::Serializer::serialize_struct(
                                __serializer,
                                "AuthDbConfig",
                                0 + 1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "db",
                                "moka",
                            )?;
                            _serde::ser::SerializeStruct::end(__serde_state)
                        }
                        AuthDbConfig::Mongodb { ref url } => {
                            let mut __serde_state = _serde::Serializer::serialize_struct(
                                __serializer,
                                "AuthDbConfig",
                                0 + 1 + 1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "db",
                                "mongodb",
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "url",
                                url,
                            )?;
                            _serde::ser::SerializeStruct::end(__serde_state)
                        }
                        AuthDbConfig::Tikv {
                            ref endpoints,
                            ref tls_config,
                            ref timeout_secs,
                        } => {
                            let mut __serde_state = _serde::Serializer::serialize_struct(
                                __serializer,
                                "AuthDbConfig",
                                0 + 1 + 1 + 1 + 1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "db",
                                "tikv",
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "endpoints",
                                endpoints,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "tls_config",
                                tls_config,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "timeout_secs",
                                timeout_secs,
                            )?;
                            _serde::ser::SerializeStruct::end(__serde_state)
                        }
                    }
                }
            }
        };
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for AuthDbConfig {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "variant identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                2u64 => _serde::__private228::Ok(__Field::__field2),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 3",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "moka" => _serde::__private228::Ok(__Field::__field0),
                                "mongodb" => _serde::__private228::Ok(__Field::__field1),
                                "tikv" => _serde::__private228::Ok(__Field::__field2),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"moka" => _serde::__private228::Ok(__Field::__field0),
                                b"mongodb" => _serde::__private228::Ok(__Field::__field1),
                                b"tikv" => _serde::__private228::Ok(__Field::__field2),
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &[
                        "moka",
                        "mongodb",
                        "tikv",
                    ];
                    let (__tag, __content) = _serde::Deserializer::deserialize_any(
                        __deserializer,
                        _serde::__private228::de::TaggedContentVisitor::<
                            __Field,
                        >::new("db", "internally tagged enum AuthDbConfig"),
                    )?;
                    let __deserializer = _serde::__private228::de::ContentDeserializer::<
                        __D::Error,
                    >::new(__content);
                    match __tag {
                        __Field::__field0 => {
                            #[allow(non_camel_case_types)]
                            #[doc(hidden)]
                            enum __Field {
                                __ignore,
                            }
                            #[doc(hidden)]
                            struct __FieldVisitor;
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(
                                    self,
                                    __value: &[u8],
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                            }
                            #[automatically_derived]
                            impl<'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::__private228::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_identifier(
                                        __deserializer,
                                        __FieldVisitor,
                                    )
                                }
                            }
                            #[doc(hidden)]
                            struct __Visitor<'de> {
                                marker: _serde::__private228::PhantomData<AuthDbConfig>,
                                lifetime: _serde::__private228::PhantomData<&'de ()>,
                            }
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = AuthDbConfig;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "struct variant AuthDbConfig::Moka",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    _: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    _serde::__private228::Ok(AuthDbConfig::Moka {})
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                        __Field,
                                    >(&mut __map)? {
                                        match __key {
                                            _ => {
                                                let _ = _serde::de::MapAccess::next_value::<
                                                    _serde::de::IgnoredAny,
                                                >(&mut __map)?;
                                            }
                                        }
                                    }
                                    _serde::__private228::Ok(AuthDbConfig::Moka {})
                                }
                            }
                            #[doc(hidden)]
                            const FIELDS: &'static [&'static str] = &[];
                            _serde::Deserializer::deserialize_any(
                                __deserializer,
                                __Visitor {
                                    marker: _serde::__private228::PhantomData::<AuthDbConfig>,
                                    lifetime: _serde::__private228::PhantomData,
                                },
                            )
                        }
                        __Field::__field1 => {
                            #[allow(non_camel_case_types)]
                            #[doc(hidden)]
                            enum __Field {
                                __field0,
                                __ignore,
                            }
                            #[doc(hidden)]
                            struct __FieldVisitor;
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        0u64 => _serde::__private228::Ok(__Field::__field0),
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        "url" => _serde::__private228::Ok(__Field::__field0),
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(
                                    self,
                                    __value: &[u8],
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        b"url" => _serde::__private228::Ok(__Field::__field0),
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                            }
                            #[automatically_derived]
                            impl<'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::__private228::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_identifier(
                                        __deserializer,
                                        __FieldVisitor,
                                    )
                                }
                            }
                            #[doc(hidden)]
                            struct __Visitor<'de> {
                                marker: _serde::__private228::PhantomData<AuthDbConfig>,
                                lifetime: _serde::__private228::PhantomData<&'de ()>,
                            }
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = AuthDbConfig;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "struct variant AuthDbConfig::Mongodb",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match _serde::de::SeqAccess::next_element::<
                                        Box<str>,
                                    >(&mut __seq)? {
                                        _serde::__private228::Some(__value) => __value,
                                        _serde::__private228::None => {
                                            return _serde::__private228::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"struct variant AuthDbConfig::Mongodb with 1 element",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private228::Ok(AuthDbConfig::Mongodb {
                                        url: __field0,
                                    })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::__private228::Option<Box<str>> = _serde::__private228::None;
                                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                        __Field,
                                    >(&mut __map)? {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::__private228::Option::is_some(&__field0) {
                                                    return _serde::__private228::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field("url"),
                                                    );
                                                }
                                                __field0 = _serde::__private228::Some(
                                                    _serde::de::MapAccess::next_value::<Box<str>>(&mut __map)?,
                                                );
                                            }
                                            _ => {
                                                let _ = _serde::de::MapAccess::next_value::<
                                                    _serde::de::IgnoredAny,
                                                >(&mut __map)?;
                                            }
                                        }
                                    }
                                    let __field0 = match __field0 {
                                        _serde::__private228::Some(__field0) => __field0,
                                        _serde::__private228::None => {
                                            _serde::__private228::de::missing_field("url")?
                                        }
                                    };
                                    _serde::__private228::Ok(AuthDbConfig::Mongodb {
                                        url: __field0,
                                    })
                                }
                            }
                            #[doc(hidden)]
                            const FIELDS: &'static [&'static str] = &["url"];
                            _serde::Deserializer::deserialize_any(
                                __deserializer,
                                __Visitor {
                                    marker: _serde::__private228::PhantomData::<AuthDbConfig>,
                                    lifetime: _serde::__private228::PhantomData,
                                },
                            )
                        }
                        __Field::__field2 => {
                            #[allow(non_camel_case_types)]
                            #[doc(hidden)]
                            enum __Field {
                                __field0,
                                __field1,
                                __field2,
                                __ignore,
                            }
                            #[doc(hidden)]
                            struct __FieldVisitor;
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        0u64 => _serde::__private228::Ok(__Field::__field0),
                                        1u64 => _serde::__private228::Ok(__Field::__field1),
                                        2u64 => _serde::__private228::Ok(__Field::__field2),
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        "endpoints" => _serde::__private228::Ok(__Field::__field0),
                                        "tls_config" => _serde::__private228::Ok(__Field::__field1),
                                        "timeout_secs" => {
                                            _serde::__private228::Ok(__Field::__field2)
                                        }
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(
                                    self,
                                    __value: &[u8],
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        b"endpoints" => _serde::__private228::Ok(__Field::__field0),
                                        b"tls_config" => _serde::__private228::Ok(__Field::__field1),
                                        b"timeout_secs" => {
                                            _serde::__private228::Ok(__Field::__field2)
                                        }
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                            }
                            #[automatically_derived]
                            impl<'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::__private228::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_identifier(
                                        __deserializer,
                                        __FieldVisitor,
                                    )
                                }
                            }
                            #[doc(hidden)]
                            struct __Visitor<'de> {
                                marker: _serde::__private228::PhantomData<AuthDbConfig>,
                                lifetime: _serde::__private228::PhantomData<&'de ()>,
                            }
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = AuthDbConfig;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "struct variant AuthDbConfig::Tikv",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match _serde::de::SeqAccess::next_element::<
                                        Box<[Box<str>]>,
                                    >(&mut __seq)? {
                                        _serde::__private228::Some(__value) => __value,
                                        _serde::__private228::None => {
                                            return _serde::__private228::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"struct variant AuthDbConfig::Tikv with 3 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field1 = match _serde::de::SeqAccess::next_element::<
                                        Option<TlsConfig>,
                                    >(&mut __seq)? {
                                        _serde::__private228::Some(__value) => __value,
                                        _serde::__private228::None => {
                                            return _serde::__private228::Err(
                                                _serde::de::Error::invalid_length(
                                                    1usize,
                                                    &"struct variant AuthDbConfig::Tikv with 3 elements",
                                                ),
                                            );
                                        }
                                    };
                                    let __field2 = match _serde::de::SeqAccess::next_element::<
                                        u64,
                                    >(&mut __seq)? {
                                        _serde::__private228::Some(__value) => __value,
                                        _serde::__private228::None => def_timeout(),
                                    };
                                    _serde::__private228::Ok(AuthDbConfig::Tikv {
                                        endpoints: __field0,
                                        tls_config: __field1,
                                        timeout_secs: __field2,
                                    })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::__private228::Option<
                                        Box<[Box<str>]>,
                                    > = _serde::__private228::None;
                                    let mut __field1: _serde::__private228::Option<
                                        Option<TlsConfig>,
                                    > = _serde::__private228::None;
                                    let mut __field2: _serde::__private228::Option<u64> = _serde::__private228::None;
                                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                        __Field,
                                    >(&mut __map)? {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::__private228::Option::is_some(&__field0) {
                                                    return _serde::__private228::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                                            "endpoints",
                                                        ),
                                                    );
                                                }
                                                __field0 = _serde::__private228::Some(
                                                    _serde::de::MapAccess::next_value::<
                                                        Box<[Box<str>]>,
                                                    >(&mut __map)?,
                                                );
                                            }
                                            __Field::__field1 => {
                                                if _serde::__private228::Option::is_some(&__field1) {
                                                    return _serde::__private228::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                                            "tls_config",
                                                        ),
                                                    );
                                                }
                                                __field1 = _serde::__private228::Some(
                                                    _serde::de::MapAccess::next_value::<
                                                        Option<TlsConfig>,
                                                    >(&mut __map)?,
                                                );
                                            }
                                            __Field::__field2 => {
                                                if _serde::__private228::Option::is_some(&__field2) {
                                                    return _serde::__private228::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                                            "timeout_secs",
                                                        ),
                                                    );
                                                }
                                                __field2 = _serde::__private228::Some(
                                                    _serde::de::MapAccess::next_value::<u64>(&mut __map)?,
                                                );
                                            }
                                            _ => {
                                                let _ = _serde::de::MapAccess::next_value::<
                                                    _serde::de::IgnoredAny,
                                                >(&mut __map)?;
                                            }
                                        }
                                    }
                                    let __field0 = match __field0 {
                                        _serde::__private228::Some(__field0) => __field0,
                                        _serde::__private228::None => {
                                            _serde::__private228::de::missing_field("endpoints")?
                                        }
                                    };
                                    let __field1 = match __field1 {
                                        _serde::__private228::Some(__field1) => __field1,
                                        _serde::__private228::None => {
                                            _serde::__private228::de::missing_field("tls_config")?
                                        }
                                    };
                                    let __field2 = match __field2 {
                                        _serde::__private228::Some(__field2) => __field2,
                                        _serde::__private228::None => def_timeout(),
                                    };
                                    _serde::__private228::Ok(AuthDbConfig::Tikv {
                                        endpoints: __field0,
                                        tls_config: __field1,
                                        timeout_secs: __field2,
                                    })
                                }
                            }
                            #[doc(hidden)]
                            const FIELDS: &'static [&'static str] = &[
                                "endpoints",
                                "tls_config",
                                "timeout_secs",
                            ];
                            _serde::Deserializer::deserialize_any(
                                __deserializer,
                                __Visitor {
                                    marker: _serde::__private228::PhantomData::<AuthDbConfig>,
                                    lifetime: _serde::__private228::PhantomData,
                                },
                            )
                        }
                    }
                }
            }
        };
        impl ::zeroize::Zeroize for AuthDbConfig {
            fn zeroize(&mut self) {
                match self {
                    #[allow(unused_variables)]
                    AuthDbConfig::Moka {} => {}
                    #[allow(unused_variables)]
                    AuthDbConfig::Mongodb { url } => url.zeroize(),
                    #[allow(unused_variables)]
                    AuthDbConfig::Tikv { endpoints, tls_config, timeout_secs } => {
                        endpoints.zeroize();
                        tls_config.zeroize();
                        timeout_secs.zeroize()
                    }
                    _ => {}
                }
            }
        }
        impl Drop for AuthDbConfig {
            fn drop(&mut self) {
                use ::zeroize::__internal::AssertZeroize;
                use ::zeroize::__internal::AssertZeroizeOnDrop;
                match self {
                    #[allow(unused_variables)]
                    AuthDbConfig::Moka {} => {}
                    #[allow(unused_variables)]
                    AuthDbConfig::Mongodb { url } => url.zeroize_or_on_drop(),
                    #[allow(unused_variables)]
                    AuthDbConfig::Tikv { endpoints, tls_config, timeout_secs } => {
                        endpoints.zeroize_or_on_drop();
                        tls_config.zeroize_or_on_drop();
                        timeout_secs.zeroize_or_on_drop()
                    }
                    _ => {}
                }
            }
        }
        #[doc(hidden)]
        impl ::zeroize::ZeroizeOnDrop for AuthDbConfig {}
        fn def_timeout() -> u64 {
            10
        }
        #[serde(tag = "db")]
        pub enum CacheConfig {
            #[default]
            #[serde(rename = "moka")]
            Moka,
            #[serde(rename = "redis")]
            Redis { url: Box<str> },
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for CacheConfig {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    CacheConfig::Moka => ::core::fmt::Formatter::write_str(f, "Moka"),
                    CacheConfig::Redis { url: __self_0 } => {
                        ::core::fmt::Formatter::debug_struct_field1_finish(
                            f,
                            "Redis",
                            "url",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CacheConfig {
            #[inline]
            fn clone(&self) -> CacheConfig {
                match self {
                    CacheConfig::Moka => CacheConfig::Moka,
                    CacheConfig::Redis { url: __self_0 } => {
                        CacheConfig::Redis {
                            url: ::core::clone::Clone::clone(__self_0),
                        }
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for CacheConfig {
            #[inline]
            fn default() -> CacheConfig {
                Self::Moka
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for CacheConfig {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        CacheConfig::Moka => {
                            let mut __struct = _serde::Serializer::serialize_struct(
                                __serializer,
                                "CacheConfig",
                                1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __struct,
                                "db",
                                "moka",
                            )?;
                            _serde::ser::SerializeStruct::end(__struct)
                        }
                        CacheConfig::Redis { ref url } => {
                            let mut __serde_state = _serde::Serializer::serialize_struct(
                                __serializer,
                                "CacheConfig",
                                0 + 1 + 1,
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "db",
                                "redis",
                            )?;
                            _serde::ser::SerializeStruct::serialize_field(
                                &mut __serde_state,
                                "url",
                                url,
                            )?;
                            _serde::ser::SerializeStruct::end(__serde_state)
                        }
                    }
                }
            }
        };
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for CacheConfig {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "variant identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::invalid_value(
                                            _serde::de::Unexpected::Unsigned(__value),
                                            &"variant index 0 <= i < 2",
                                        ),
                                    )
                                }
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "moka" => _serde::__private228::Ok(__Field::__field0),
                                "redis" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"moka" => _serde::__private228::Ok(__Field::__field0),
                                b"redis" => _serde::__private228::Ok(__Field::__field1),
                                _ => {
                                    let __value = &_serde::__private228::from_utf8_lossy(
                                        __value,
                                    );
                                    _serde::__private228::Err(
                                        _serde::de::Error::unknown_variant(__value, VARIANTS),
                                    )
                                }
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &["moka", "redis"];
                    let (__tag, __content) = _serde::Deserializer::deserialize_any(
                        __deserializer,
                        _serde::__private228::de::TaggedContentVisitor::<
                            __Field,
                        >::new("db", "internally tagged enum CacheConfig"),
                    )?;
                    let __deserializer = _serde::__private228::de::ContentDeserializer::<
                        __D::Error,
                    >::new(__content);
                    match __tag {
                        __Field::__field0 => {
                            _serde::Deserializer::deserialize_any(
                                __deserializer,
                                _serde::__private228::de::InternallyTaggedUnitVisitor::new(
                                    "CacheConfig",
                                    "Moka",
                                ),
                            )?;
                            _serde::__private228::Ok(CacheConfig::Moka)
                        }
                        __Field::__field1 => {
                            #[allow(non_camel_case_types)]
                            #[doc(hidden)]
                            enum __Field {
                                __field0,
                                __ignore,
                            }
                            #[doc(hidden)]
                            struct __FieldVisitor;
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                type Value = __Field;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "field identifier",
                                    )
                                }
                                fn visit_u64<__E>(
                                    self,
                                    __value: u64,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        0u64 => _serde::__private228::Ok(__Field::__field0),
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_str<__E>(
                                    self,
                                    __value: &str,
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        "url" => _serde::__private228::Ok(__Field::__field0),
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(
                                    self,
                                    __value: &[u8],
                                ) -> _serde::__private228::Result<Self::Value, __E>
                                where
                                    __E: _serde::de::Error,
                                {
                                    match __value {
                                        b"url" => _serde::__private228::Ok(__Field::__field0),
                                        _ => _serde::__private228::Ok(__Field::__ignore),
                                    }
                                }
                            }
                            #[automatically_derived]
                            impl<'de> _serde::Deserialize<'de> for __Field {
                                #[inline]
                                fn deserialize<__D>(
                                    __deserializer: __D,
                                ) -> _serde::__private228::Result<Self, __D::Error>
                                where
                                    __D: _serde::Deserializer<'de>,
                                {
                                    _serde::Deserializer::deserialize_identifier(
                                        __deserializer,
                                        __FieldVisitor,
                                    )
                                }
                            }
                            #[doc(hidden)]
                            struct __Visitor<'de> {
                                marker: _serde::__private228::PhantomData<CacheConfig>,
                                lifetime: _serde::__private228::PhantomData<&'de ()>,
                            }
                            #[automatically_derived]
                            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                type Value = CacheConfig;
                                fn expecting(
                                    &self,
                                    __formatter: &mut _serde::__private228::Formatter,
                                ) -> _serde::__private228::fmt::Result {
                                    _serde::__private228::Formatter::write_str(
                                        __formatter,
                                        "struct variant CacheConfig::Redis",
                                    )
                                }
                                #[inline]
                                fn visit_seq<__A>(
                                    self,
                                    mut __seq: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::SeqAccess<'de>,
                                {
                                    let __field0 = match _serde::de::SeqAccess::next_element::<
                                        Box<str>,
                                    >(&mut __seq)? {
                                        _serde::__private228::Some(__value) => __value,
                                        _serde::__private228::None => {
                                            return _serde::__private228::Err(
                                                _serde::de::Error::invalid_length(
                                                    0usize,
                                                    &"struct variant CacheConfig::Redis with 1 element",
                                                ),
                                            );
                                        }
                                    };
                                    _serde::__private228::Ok(CacheConfig::Redis {
                                        url: __field0,
                                    })
                                }
                                #[inline]
                                fn visit_map<__A>(
                                    self,
                                    mut __map: __A,
                                ) -> _serde::__private228::Result<Self::Value, __A::Error>
                                where
                                    __A: _serde::de::MapAccess<'de>,
                                {
                                    let mut __field0: _serde::__private228::Option<Box<str>> = _serde::__private228::None;
                                    while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                        __Field,
                                    >(&mut __map)? {
                                        match __key {
                                            __Field::__field0 => {
                                                if _serde::__private228::Option::is_some(&__field0) {
                                                    return _serde::__private228::Err(
                                                        <__A::Error as _serde::de::Error>::duplicate_field("url"),
                                                    );
                                                }
                                                __field0 = _serde::__private228::Some(
                                                    _serde::de::MapAccess::next_value::<Box<str>>(&mut __map)?,
                                                );
                                            }
                                            _ => {
                                                let _ = _serde::de::MapAccess::next_value::<
                                                    _serde::de::IgnoredAny,
                                                >(&mut __map)?;
                                            }
                                        }
                                    }
                                    let __field0 = match __field0 {
                                        _serde::__private228::Some(__field0) => __field0,
                                        _serde::__private228::None => {
                                            _serde::__private228::de::missing_field("url")?
                                        }
                                    };
                                    _serde::__private228::Ok(CacheConfig::Redis {
                                        url: __field0,
                                    })
                                }
                            }
                            #[doc(hidden)]
                            const FIELDS: &'static [&'static str] = &["url"];
                            _serde::Deserializer::deserialize_any(
                                __deserializer,
                                __Visitor {
                                    marker: _serde::__private228::PhantomData::<CacheConfig>,
                                    lifetime: _serde::__private228::PhantomData,
                                },
                            )
                        }
                    }
                }
            }
        };
        impl ::zeroize::Zeroize for CacheConfig {
            fn zeroize(&mut self) {
                match self {
                    #[allow(unused_variables)]
                    CacheConfig::Moka => {}
                    #[allow(unused_variables)]
                    CacheConfig::Redis { url } => url.zeroize(),
                    _ => {}
                }
            }
        }
        impl Drop for CacheConfig {
            fn drop(&mut self) {
                use ::zeroize::__internal::AssertZeroize;
                use ::zeroize::__internal::AssertZeroizeOnDrop;
                match self {
                    #[allow(unused_variables)]
                    CacheConfig::Moka => {}
                    #[allow(unused_variables)]
                    CacheConfig::Redis { url } => url.zeroize_or_on_drop(),
                    _ => {}
                }
            }
        }
        #[doc(hidden)]
        impl ::zeroize::ZeroizeOnDrop for CacheConfig {}
        impl Default for AuthDbConfig {
            fn default() -> Self {
                Self::Mongodb {
                    url: String::new().into_boxed_str(),
                }
            }
        }
        pub struct TlsConfig {
            pub ca_path: Box<str>,
            pub cert_path: Box<str>,
            pub key_path: Box<str>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for TlsConfig {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "TlsConfig",
                    "ca_path",
                    &self.ca_path,
                    "cert_path",
                    &self.cert_path,
                    "key_path",
                    &&self.key_path,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for TlsConfig {
            #[inline]
            fn clone(&self) -> TlsConfig {
                TlsConfig {
                    ca_path: ::core::clone::Clone::clone(&self.ca_path),
                    cert_path: ::core::clone::Clone::clone(&self.cert_path),
                    key_path: ::core::clone::Clone::clone(&self.key_path),
                }
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for TlsConfig {
            #[inline]
            fn default() -> TlsConfig {
                TlsConfig {
                    ca_path: ::core::default::Default::default(),
                    cert_path: ::core::default::Default::default(),
                    key_path: ::core::default::Default::default(),
                }
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for TlsConfig {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "TlsConfig",
                        false as usize + 1 + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "ca_path",
                        &self.ca_path,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "cert_path",
                        &self.cert_path,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "key_path",
                        &self.key_path,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl<'de> _serde::Deserialize<'de> for TlsConfig {
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private228::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    #[allow(non_camel_case_types)]
                    #[doc(hidden)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __ignore,
                    }
                    #[doc(hidden)]
                    struct __FieldVisitor;
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "field identifier",
                            )
                        }
                        fn visit_u64<__E>(
                            self,
                            __value: u64,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                0u64 => _serde::__private228::Ok(__Field::__field0),
                                1u64 => _serde::__private228::Ok(__Field::__field1),
                                2u64 => _serde::__private228::Ok(__Field::__field2),
                                _ => _serde::__private228::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<__E>(
                            self,
                            __value: &str,
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                "ca_path" => _serde::__private228::Ok(__Field::__field0),
                                "cert_path" => _serde::__private228::Ok(__Field::__field1),
                                "key_path" => _serde::__private228::Ok(__Field::__field2),
                                _ => _serde::__private228::Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<__E>(
                            self,
                            __value: &[u8],
                        ) -> _serde::__private228::Result<Self::Value, __E>
                        where
                            __E: _serde::de::Error,
                        {
                            match __value {
                                b"ca_path" => _serde::__private228::Ok(__Field::__field0),
                                b"cert_path" => _serde::__private228::Ok(__Field::__field1),
                                b"key_path" => _serde::__private228::Ok(__Field::__field2),
                                _ => _serde::__private228::Ok(__Field::__ignore),
                            }
                        }
                    }
                    #[automatically_derived]
                    impl<'de> _serde::Deserialize<'de> for __Field {
                        #[inline]
                        fn deserialize<__D>(
                            __deserializer: __D,
                        ) -> _serde::__private228::Result<Self, __D::Error>
                        where
                            __D: _serde::Deserializer<'de>,
                        {
                            _serde::Deserializer::deserialize_identifier(
                                __deserializer,
                                __FieldVisitor,
                            )
                        }
                    }
                    #[doc(hidden)]
                    struct __Visitor<'de> {
                        marker: _serde::__private228::PhantomData<TlsConfig>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = TlsConfig;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct TlsConfig",
                            )
                        }
                        #[inline]
                        fn visit_seq<__A>(
                            self,
                            mut __seq: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::SeqAccess<'de>,
                        {
                            let __field0 = match _serde::de::SeqAccess::next_element::<
                                Box<str>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct TlsConfig with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field1 = match _serde::de::SeqAccess::next_element::<
                                Box<str>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            1usize,
                                            &"struct TlsConfig with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                Box<str>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct TlsConfig with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(TlsConfig {
                                ca_path: __field0,
                                cert_path: __field1,
                                key_path: __field2,
                            })
                        }
                        #[inline]
                        fn visit_map<__A>(
                            self,
                            mut __map: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::MapAccess<'de>,
                        {
                            let mut __field0: _serde::__private228::Option<Box<str>> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<Box<str>> = _serde::__private228::None;
                            let mut __field2: _serde::__private228::Option<Box<str>> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "ca_path",
                                                ),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<Box<str>>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "cert_path",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<Box<str>>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private228::Option::is_some(&__field2) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "key_path",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<Box<str>>(&mut __map)?,
                                        );
                                    }
                                    _ => {
                                        let _ = _serde::de::MapAccess::next_value::<
                                            _serde::de::IgnoredAny,
                                        >(&mut __map)?;
                                    }
                                }
                            }
                            let __field0 = match __field0 {
                                _serde::__private228::Some(__field0) => __field0,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("ca_path")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("cert_path")?
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private228::Some(__field2) => __field2,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("key_path")?
                                }
                            };
                            _serde::__private228::Ok(TlsConfig {
                                ca_path: __field0,
                                cert_path: __field1,
                                key_path: __field2,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "ca_path",
                        "cert_path",
                        "key_path",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "TlsConfig",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<TlsConfig>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
                }
            }
        };
        impl ::zeroize::Zeroize for TlsConfig {
            fn zeroize(&mut self) {
                match self {
                    #[allow(unused_variables)]
                    TlsConfig { ca_path, cert_path, key_path } => {
                        ca_path.zeroize();
                        cert_path.zeroize();
                        key_path.zeroize()
                    }
                    _ => {}
                }
            }
        }
        impl Drop for TlsConfig {
            fn drop(&mut self) {
                use ::zeroize::__internal::AssertZeroize;
                use ::zeroize::__internal::AssertZeroizeOnDrop;
                match self {
                    #[allow(unused_variables)]
                    TlsConfig { ca_path, cert_path, key_path } => {
                        ca_path.zeroize_or_on_drop();
                        cert_path.zeroize_or_on_drop();
                        key_path.zeroize_or_on_drop()
                    }
                    _ => {}
                }
            }
        }
        #[doc(hidden)]
        impl ::zeroize::ZeroizeOnDrop for TlsConfig {}
    }
    pub mod error {
        use actix_web::http::StatusCode;
        use base64::DecodeError;
        use mongodb::error::Error as MongoDBError;
        use rand::rand_core::OsError;
        use redis::RedisError;
        use thiserror::Error;
        use argon2::password_hash::Error as ArgonErr;
        use serde_json::Error as SerdeError;
        use std::io::Error as StdError;
        use tikv_client::Error as TikvError;
        use tokio::task::JoinError;
        pub enum ServerError {
            #[error(transparent)]
            Serde(#[from] SerdeError),
            #[error(transparent)]
            Base64(#[from] DecodeError),
            #[error(transparent)]
            TokioJoinError(#[from] JoinError),
            #[error(transparent)]
            Std(#[from] StdError),
            #[error(transparent)]
            Tikv(#[from] TikvError),
            #[error(transparent)]
            MongoDB(#[from] MongoDBError),
            #[error(transparent)]
            RedisDBError(#[from] RedisError),
            #[error("Failed to convert OS String to String")]
            StringConvertErr,
            #[error("Tried to retry many times but failed")]
            RetryFailed,
            #[error("Argon Hashing Error")]
            ArgonErr(ArgonErr),
            #[error(transparent)]
            RngErr(#[from] OsError),
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for ServerError {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    ServerError::Serde(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Serde",
                            &__self_0,
                        )
                    }
                    ServerError::Base64(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Base64",
                            &__self_0,
                        )
                    }
                    ServerError::TokioJoinError(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "TokioJoinError",
                            &__self_0,
                        )
                    }
                    ServerError::Std(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Std",
                            &__self_0,
                        )
                    }
                    ServerError::Tikv(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Tikv",
                            &__self_0,
                        )
                    }
                    ServerError::MongoDB(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "MongoDB",
                            &__self_0,
                        )
                    }
                    ServerError::RedisDBError(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "RedisDBError",
                            &__self_0,
                        )
                    }
                    ServerError::StringConvertErr => {
                        ::core::fmt::Formatter::write_str(f, "StringConvertErr")
                    }
                    ServerError::RetryFailed => {
                        ::core::fmt::Formatter::write_str(f, "RetryFailed")
                    }
                    ServerError::ArgonErr(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ArgonErr",
                            &__self_0,
                        )
                    }
                    ServerError::RngErr(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "RngErr",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl ::thiserror::__private17::Error for ServerError {
            fn source(
                &self,
            ) -> ::core::option::Option<
                &(dyn ::thiserror::__private17::Error + 'static),
            > {
                use ::thiserror::__private17::AsDynError as _;
                #[allow(deprecated)]
                match self {
                    ServerError::Serde { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                    ServerError::Base64 { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                    ServerError::TokioJoinError { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                    ServerError::Std { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                    ServerError::Tikv { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                    ServerError::MongoDB { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                    ServerError::RedisDBError { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                    ServerError::StringConvertErr { .. } => ::core::option::Option::None,
                    ServerError::RetryFailed { .. } => ::core::option::Option::None,
                    ServerError::ArgonErr { .. } => ::core::option::Option::None,
                    ServerError::RngErr { 0: transparent } => {
                        ::thiserror::__private17::Error::source(
                            transparent.as_dyn_error(),
                        )
                    }
                }
            }
        }
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl ::core::fmt::Display for ServerError {
            fn fmt(
                &self,
                __formatter: &mut ::core::fmt::Formatter,
            ) -> ::core::fmt::Result {
                #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
                match self {
                    ServerError::Serde(_0) => ::core::fmt::Display::fmt(_0, __formatter),
                    ServerError::Base64(_0) => ::core::fmt::Display::fmt(_0, __formatter),
                    ServerError::TokioJoinError(_0) => {
                        ::core::fmt::Display::fmt(_0, __formatter)
                    }
                    ServerError::Std(_0) => ::core::fmt::Display::fmt(_0, __formatter),
                    ServerError::Tikv(_0) => ::core::fmt::Display::fmt(_0, __formatter),
                    ServerError::MongoDB(_0) => {
                        ::core::fmt::Display::fmt(_0, __formatter)
                    }
                    ServerError::RedisDBError(_0) => {
                        ::core::fmt::Display::fmt(_0, __formatter)
                    }
                    ServerError::StringConvertErr {} => {
                        __formatter.write_str("Failed to convert OS String to String")
                    }
                    ServerError::RetryFailed {} => {
                        __formatter.write_str("Tried to retry many times but failed")
                    }
                    ServerError::ArgonErr(_0) => {
                        __formatter.write_str("Argon Hashing Error")
                    }
                    ServerError::RngErr(_0) => ::core::fmt::Display::fmt(_0, __formatter),
                }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<SerdeError> for ServerError {
            fn from(source: SerdeError) -> Self {
                ServerError::Serde { 0: source }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<DecodeError> for ServerError {
            fn from(source: DecodeError) -> Self {
                ServerError::Base64 { 0: source }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<JoinError> for ServerError {
            fn from(source: JoinError) -> Self {
                ServerError::TokioJoinError {
                    0: source,
                }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<StdError> for ServerError {
            fn from(source: StdError) -> Self {
                ServerError::Std { 0: source }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<TikvError> for ServerError {
            fn from(source: TikvError) -> Self {
                ServerError::Tikv { 0: source }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<MongoDBError> for ServerError {
            fn from(source: MongoDBError) -> Self {
                ServerError::MongoDB { 0: source }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<RedisError> for ServerError {
            fn from(source: RedisError) -> Self {
                ServerError::RedisDBError {
                    0: source,
                }
            }
        }
        #[allow(
            deprecated,
            unused_qualifications,
            clippy::elidable_lifetime_names,
            clippy::needless_lifetimes,
        )]
        #[automatically_derived]
        impl ::core::convert::From<OsError> for ServerError {
            fn from(source: OsError) -> Self {
                ServerError::RngErr { 0: source }
            }
        }
        impl actix_web::error::ResponseError for ServerError {
            fn status_code(&self) -> actix_web::http::StatusCode {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        pub type Returns<T> = Result<T, ServerError>;
    }
    const VERSION: u16 = 1;
    pub struct Config {
        pub version: u16,
        #[serde(default = "def_bind")]
        pub binds: Vec<(String, u16)>,
        pub admin_pass_hash: Option<String>,
        pub llama: LlamaConfiguration,
        pub authentication: Authentication,
        pub database: DatabaseConfig,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Config {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "version",
                "binds",
                "admin_pass_hash",
                "llama",
                "authentication",
                "database",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.version,
                &self.binds,
                &self.admin_pass_hash,
                &self.llama,
                &self.authentication,
                &&self.database,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Config",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Config {
        #[inline]
        fn clone(&self) -> Config {
            Config {
                version: ::core::clone::Clone::clone(&self.version),
                binds: ::core::clone::Clone::clone(&self.binds),
                admin_pass_hash: ::core::clone::Clone::clone(&self.admin_pass_hash),
                llama: ::core::clone::Clone::clone(&self.llama),
                authentication: ::core::clone::Clone::clone(&self.authentication),
                database: ::core::clone::Clone::clone(&self.database),
            }
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Config {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Config",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "version",
                    &self.version,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "binds",
                    &self.binds,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "admin_pass_hash",
                    &self.admin_pass_hash,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "llama",
                    &self.llama,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "authentication",
                    &self.authentication,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "database",
                    &self.database,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Config {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            4u64 => _serde::__private228::Ok(__Field::__field4),
                            5u64 => _serde::__private228::Ok(__Field::__field5),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "version" => _serde::__private228::Ok(__Field::__field0),
                            "binds" => _serde::__private228::Ok(__Field::__field1),
                            "admin_pass_hash" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            "llama" => _serde::__private228::Ok(__Field::__field3),
                            "authentication" => {
                                _serde::__private228::Ok(__Field::__field4)
                            }
                            "database" => _serde::__private228::Ok(__Field::__field5),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"version" => _serde::__private228::Ok(__Field::__field0),
                            b"binds" => _serde::__private228::Ok(__Field::__field1),
                            b"admin_pass_hash" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"llama" => _serde::__private228::Ok(__Field::__field3),
                            b"authentication" => {
                                _serde::__private228::Ok(__Field::__field4)
                            }
                            b"database" => _serde::__private228::Ok(__Field::__field5),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<Config>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Config;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct Config",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            u16,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Config with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Vec<(String, u16)>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => def_bind(),
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Config with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            LlamaConfiguration,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Config with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            Authentication,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Config with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            DatabaseConfig,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Config with 6 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(Config {
                            version: __field0,
                            binds: __field1,
                            admin_pass_hash: __field2,
                            llama: __field3,
                            authentication: __field4,
                            database: __field5,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<u16> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<
                            Vec<(String, u16)>,
                        > = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<Option<String>> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<
                            LlamaConfiguration,
                        > = _serde::__private228::None;
                        let mut __field4: _serde::__private228::Option<Authentication> = _serde::__private228::None;
                        let mut __field5: _serde::__private228::Option<DatabaseConfig> = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "version",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<u16>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("binds"),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<(String, u16)>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "admin_pass_hash",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("llama"),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            LlamaConfiguration,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private228::Option::is_some(&__field4) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "authentication",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Authentication,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private228::Option::is_some(&__field5) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "database",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            DatabaseConfig,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("version")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => def_bind(),
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("admin_pass_hash")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("llama")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private228::Some(__field4) => __field4,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("authentication")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private228::Some(__field5) => __field5,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("database")?
                            }
                        };
                        _serde::__private228::Ok(Config {
                            version: __field0,
                            binds: __field1,
                            admin_pass_hash: __field2,
                            llama: __field3,
                            authentication: __field4,
                            database: __field5,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "version",
                    "binds",
                    "admin_pass_hash",
                    "llama",
                    "authentication",
                    "database",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Config",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<Config>,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    impl ::zeroize::Zeroize for Config {
        fn zeroize(&mut self) {
            match self {
                #[allow(unused_variables)]
                Config {
                    version,
                    binds,
                    admin_pass_hash,
                    llama,
                    authentication,
                    database,
                } => {
                    version.zeroize();
                    binds.zeroize();
                    admin_pass_hash.zeroize();
                    llama.zeroize();
                    authentication.zeroize();
                    database.zeroize()
                }
                _ => {}
            }
        }
    }
    impl Drop for Config {
        fn drop(&mut self) {
            use ::zeroize::__internal::AssertZeroize;
            use ::zeroize::__internal::AssertZeroizeOnDrop;
            match self {
                #[allow(unused_variables)]
                Config {
                    version,
                    binds,
                    admin_pass_hash,
                    llama,
                    authentication,
                    database,
                } => {
                    version.zeroize_or_on_drop();
                    binds.zeroize_or_on_drop();
                    admin_pass_hash.zeroize_or_on_drop();
                    llama.zeroize_or_on_drop();
                    authentication.zeroize_or_on_drop();
                    database.zeroize_or_on_drop()
                }
                _ => {}
            }
        }
    }
    #[doc(hidden)]
    impl ::zeroize::ZeroizeOnDrop for Config {}
    fn def_bind() -> Vec<(String, u16)> {
        <[_]>::into_vec(
            ::alloc::boxed::box_new([
                ("0.0.0.0".to_string(), 3000),
                ("localhost".to_string(), 3000),
            ]),
        )
    }
    pub struct LlamaConfiguration {
        pub models: HashMap<Box<str>, LlamaServer>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for LlamaConfiguration {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "LlamaConfiguration",
                "models",
                &&self.models,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LlamaConfiguration {
        #[inline]
        fn clone(&self) -> LlamaConfiguration {
            LlamaConfiguration {
                models: ::core::clone::Clone::clone(&self.models),
            }
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for LlamaConfiguration {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "LlamaConfiguration",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "models",
                    &self.models,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[automatically_derived]
    impl ::core::default::Default for LlamaConfiguration {
        #[inline]
        fn default() -> LlamaConfiguration {
            LlamaConfiguration {
                models: ::core::default::Default::default(),
            }
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for LlamaConfiguration {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "models" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"models" => _serde::__private228::Ok(__Field::__field0),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<LlamaConfiguration>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = LlamaConfiguration;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct LlamaConfiguration",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            HashMap<Box<str>, LlamaServer>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct LlamaConfiguration with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(LlamaConfiguration {
                            models: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<
                            HashMap<Box<str>, LlamaServer>,
                        > = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("models"),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<Box<str>, LlamaServer>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("models")?
                            }
                        };
                        _serde::__private228::Ok(LlamaConfiguration {
                            models: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["models"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "LlamaConfiguration",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<LlamaConfiguration>,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    impl Zeroize for LlamaConfiguration {
        fn zeroize(&mut self) {
            self.models
                .iter_mut()
                .for_each(|(_, v)| {
                    v.zeroize();
                });
            self.models.clear();
        }
    }
    impl ZeroizeOnDrop for LlamaConfiguration {}
    pub struct LlamaServer {
        pub name: Box<str>,
        pub url: Box<str>,
        pub capabilities: Capabilities,
        pub apikey: Option<Box<str>>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for LlamaServer {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "LlamaServer",
                "name",
                &self.name,
                "url",
                &self.url,
                "capabilities",
                &self.capabilities,
                "apikey",
                &&self.apikey,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LlamaServer {
        #[inline]
        fn clone(&self) -> LlamaServer {
            LlamaServer {
                name: ::core::clone::Clone::clone(&self.name),
                url: ::core::clone::Clone::clone(&self.url),
                capabilities: ::core::clone::Clone::clone(&self.capabilities),
                apikey: ::core::clone::Clone::clone(&self.apikey),
            }
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for LlamaServer {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "LlamaServer",
                    false as usize + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "name",
                    &self.name,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "url",
                    &self.url,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "capabilities",
                    &self.capabilities,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "apikey",
                    &self.apikey,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for LlamaServer {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            2u64 => _serde::__private228::Ok(__Field::__field2),
                            3u64 => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "name" => _serde::__private228::Ok(__Field::__field0),
                            "url" => _serde::__private228::Ok(__Field::__field1),
                            "capabilities" => _serde::__private228::Ok(__Field::__field2),
                            "apikey" => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"name" => _serde::__private228::Ok(__Field::__field0),
                            b"url" => _serde::__private228::Ok(__Field::__field1),
                            b"capabilities" => {
                                _serde::__private228::Ok(__Field::__field2)
                            }
                            b"apikey" => _serde::__private228::Ok(__Field::__field3),
                            _ => _serde::__private228::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private228::PhantomData<LlamaServer>,
                    lifetime: _serde::__private228::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = LlamaServer;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "struct LlamaServer",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Box<str>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct LlamaServer with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Box<str>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct LlamaServer with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Capabilities,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct LlamaServer with 4 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            Option<Box<str>>,
                        >(&mut __seq)? {
                            _serde::__private228::Some(__value) => __value,
                            _serde::__private228::None => {
                                return _serde::__private228::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct LlamaServer with 4 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private228::Ok(LlamaServer {
                            name: __field0,
                            url: __field1,
                            capabilities: __field2,
                            apikey: __field3,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private228::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private228::Option<Box<str>> = _serde::__private228::None;
                        let mut __field1: _serde::__private228::Option<Box<str>> = _serde::__private228::None;
                        let mut __field2: _serde::__private228::Option<Capabilities> = _serde::__private228::None;
                        let mut __field3: _serde::__private228::Option<
                            Option<Box<str>>,
                        > = _serde::__private228::None;
                        while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private228::Option::is_some(&__field0) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                        );
                                    }
                                    __field0 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Box<str>>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private228::Option::is_some(&__field1) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("url"),
                                        );
                                    }
                                    __field1 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<Box<str>>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private228::Option::is_some(&__field2) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "capabilities",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Capabilities,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private228::Option::is_some(&__field3) {
                                        return _serde::__private228::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("apikey"),
                                        );
                                    }
                                    __field3 = _serde::__private228::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<Box<str>>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private228::Some(__field0) => __field0,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("name")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private228::Some(__field1) => __field1,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("url")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private228::Some(__field2) => __field2,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("capabilities")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private228::Some(__field3) => __field3,
                            _serde::__private228::None => {
                                _serde::__private228::de::missing_field("apikey")?
                            }
                        };
                        _serde::__private228::Ok(LlamaServer {
                            name: __field0,
                            url: __field1,
                            capabilities: __field2,
                            apikey: __field3,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "name",
                    "url",
                    "capabilities",
                    "apikey",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "LlamaServer",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private228::PhantomData::<LlamaServer>,
                        lifetime: _serde::__private228::PhantomData,
                    },
                )
            }
        }
    };
    impl ::zeroize::Zeroize for LlamaServer {
        fn zeroize(&mut self) {
            match self {
                #[allow(unused_variables)]
                LlamaServer { name, url, capabilities, apikey } => {
                    name.zeroize();
                    url.zeroize();
                    capabilities.zeroize();
                    apikey.zeroize()
                }
                _ => {}
            }
        }
    }
    impl Drop for LlamaServer {
        fn drop(&mut self) {
            use ::zeroize::__internal::AssertZeroize;
            use ::zeroize::__internal::AssertZeroizeOnDrop;
            match self {
                #[allow(unused_variables)]
                LlamaServer { name, url, capabilities, apikey } => {
                    name.zeroize_or_on_drop();
                    url.zeroize_or_on_drop();
                    capabilities.zeroize_or_on_drop();
                    apikey.zeroize_or_on_drop()
                }
                _ => {}
            }
        }
    }
    #[doc(hidden)]
    impl ::zeroize::ZeroizeOnDrop for LlamaServer {}
    pub enum ModelFlag {
        Image,
        Audio,
        Files,
    }
    impl ModelFlag {
        pub fn into_int(self) -> u16 {
            match self {
                Self::Image => 1,
                Self::Audio => 2,
                Self::Files => 4,
            }
        }
    }
    #[repr(transparent)]
    pub struct Capabilities(pub u16);
    #[automatically_derived]
    impl ::core::fmt::Debug for Capabilities {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(
                f,
                "Capabilities",
                &&self.0,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Capabilities {
        #[inline]
        fn clone(&self) -> Capabilities {
            Capabilities(::core::clone::Clone::clone(&self.0))
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for Capabilities {
        #[inline]
        fn default() -> Capabilities {
            Capabilities(::core::default::Default::default())
        }
    }
    impl ::zeroize::Zeroize for Capabilities {
        fn zeroize(&mut self) {
            match self {
                #[allow(unused_variables)]
                Capabilities(__zeroize_field_0) => __zeroize_field_0.zeroize(),
                _ => {}
            }
        }
    }
    impl Drop for Capabilities {
        fn drop(&mut self) {
            use ::zeroize::__internal::AssertZeroize;
            use ::zeroize::__internal::AssertZeroizeOnDrop;
            match self {
                #[allow(unused_variables)]
                Capabilities(__zeroize_field_0) => __zeroize_field_0.zeroize_or_on_drop(),
                _ => {}
            }
        }
    }
    #[doc(hidden)]
    impl ::zeroize::ZeroizeOnDrop for Capabilities {}
    impl Serialize for Capabilities {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for Capabilities {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(Self(u16::deserialize(deserializer)?))
        }
    }
    impl Capabilities {
        pub fn add(&mut self, flag: ModelFlag) {
            self.0 |= flag.into_int();
        }
        pub fn has(&self, flag: ModelFlag) -> bool {
            (self.0 & flag.into_int()) > 0
        }
    }
    #[serde(tag = "kind")]
    pub enum Authentication {
        OpenToAll,
        Account { registration_allowed: bool, max_memory: u32, time_cost: u32 },
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Authentication {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Authentication::OpenToAll => {
                    ::core::fmt::Formatter::write_str(f, "OpenToAll")
                }
                Authentication::Account {
                    registration_allowed: __self_0,
                    max_memory: __self_1,
                    time_cost: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "Account",
                        "registration_allowed",
                        __self_0,
                        "max_memory",
                        __self_1,
                        "time_cost",
                        &__self_2,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Authentication {
        #[inline]
        fn clone(&self) -> Authentication {
            match self {
                Authentication::OpenToAll => Authentication::OpenToAll,
                Authentication::Account {
                    registration_allowed: __self_0,
                    max_memory: __self_1,
                    time_cost: __self_2,
                } => {
                    Authentication::Account {
                        registration_allowed: ::core::clone::Clone::clone(__self_0),
                        max_memory: ::core::clone::Clone::clone(__self_1),
                        time_cost: ::core::clone::Clone::clone(__self_2),
                    }
                }
            }
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Authentication {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private228::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    Authentication::OpenToAll => {
                        let mut __struct = _serde::Serializer::serialize_struct(
                            __serializer,
                            "Authentication",
                            1,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __struct,
                            "kind",
                            "OpenToAll",
                        )?;
                        _serde::ser::SerializeStruct::end(__struct)
                    }
                    Authentication::Account {
                        ref registration_allowed,
                        ref max_memory,
                        ref time_cost,
                    } => {
                        let mut __serde_state = _serde::Serializer::serialize_struct(
                            __serializer,
                            "Authentication",
                            0 + 1 + 1 + 1 + 1,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "kind",
                            "Account",
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "registration_allowed",
                            registration_allowed,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "max_memory",
                            max_memory,
                        )?;
                        _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "time_cost",
                            time_cost,
                        )?;
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Authentication {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private228::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private228::Formatter,
                    ) -> _serde::__private228::fmt::Result {
                        _serde::__private228::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private228::Ok(__Field::__field0),
                            1u64 => _serde::__private228::Ok(__Field::__field1),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 2",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "OpenToAll" => _serde::__private228::Ok(__Field::__field0),
                            "Account" => _serde::__private228::Ok(__Field::__field1),
                            _ => {
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private228::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"OpenToAll" => _serde::__private228::Ok(__Field::__field0),
                            b"Account" => _serde::__private228::Ok(__Field::__field1),
                            _ => {
                                let __value = &_serde::__private228::from_utf8_lossy(
                                    __value,
                                );
                                _serde::__private228::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private228::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &["OpenToAll", "Account"];
                let (__tag, __content) = _serde::Deserializer::deserialize_any(
                    __deserializer,
                    _serde::__private228::de::TaggedContentVisitor::<
                        __Field,
                    >::new("kind", "internally tagged enum Authentication"),
                )?;
                let __deserializer = _serde::__private228::de::ContentDeserializer::<
                    __D::Error,
                >::new(__content);
                match __tag {
                    __Field::__field0 => {
                        _serde::Deserializer::deserialize_any(
                            __deserializer,
                            _serde::__private228::de::InternallyTaggedUnitVisitor::new(
                                "Authentication",
                                "OpenToAll",
                            ),
                        )?;
                        _serde::__private228::Ok(Authentication::OpenToAll)
                    }
                    __Field::__field1 => {
                        #[allow(non_camel_case_types)]
                        #[doc(hidden)]
                        enum __Field {
                            __field0,
                            __field1,
                            __field2,
                            __ignore,
                        }
                        #[doc(hidden)]
                        struct __FieldVisitor;
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                            type Value = __Field;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private228::Formatter,
                            ) -> _serde::__private228::fmt::Result {
                                _serde::__private228::Formatter::write_str(
                                    __formatter,
                                    "field identifier",
                                )
                            }
                            fn visit_u64<__E>(
                                self,
                                __value: u64,
                            ) -> _serde::__private228::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                            {
                                match __value {
                                    0u64 => _serde::__private228::Ok(__Field::__field0),
                                    1u64 => _serde::__private228::Ok(__Field::__field1),
                                    2u64 => _serde::__private228::Ok(__Field::__field2),
                                    _ => _serde::__private228::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_str<__E>(
                                self,
                                __value: &str,
                            ) -> _serde::__private228::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                            {
                                match __value {
                                    "registration_allowed" => {
                                        _serde::__private228::Ok(__Field::__field0)
                                    }
                                    "max_memory" => _serde::__private228::Ok(__Field::__field1),
                                    "time_cost" => _serde::__private228::Ok(__Field::__field2),
                                    _ => _serde::__private228::Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(
                                self,
                                __value: &[u8],
                            ) -> _serde::__private228::Result<Self::Value, __E>
                            where
                                __E: _serde::de::Error,
                            {
                                match __value {
                                    b"registration_allowed" => {
                                        _serde::__private228::Ok(__Field::__field0)
                                    }
                                    b"max_memory" => _serde::__private228::Ok(__Field::__field1),
                                    b"time_cost" => _serde::__private228::Ok(__Field::__field2),
                                    _ => _serde::__private228::Ok(__Field::__ignore),
                                }
                            }
                        }
                        #[automatically_derived]
                        impl<'de> _serde::Deserialize<'de> for __Field {
                            #[inline]
                            fn deserialize<__D>(
                                __deserializer: __D,
                            ) -> _serde::__private228::Result<Self, __D::Error>
                            where
                                __D: _serde::Deserializer<'de>,
                            {
                                _serde::Deserializer::deserialize_identifier(
                                    __deserializer,
                                    __FieldVisitor,
                                )
                            }
                        }
                        #[doc(hidden)]
                        struct __Visitor<'de> {
                            marker: _serde::__private228::PhantomData<Authentication>,
                            lifetime: _serde::__private228::PhantomData<&'de ()>,
                        }
                        #[automatically_derived]
                        impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                            type Value = Authentication;
                            fn expecting(
                                &self,
                                __formatter: &mut _serde::__private228::Formatter,
                            ) -> _serde::__private228::fmt::Result {
                                _serde::__private228::Formatter::write_str(
                                    __formatter,
                                    "struct variant Authentication::Account",
                                )
                            }
                            #[inline]
                            fn visit_seq<__A>(
                                self,
                                mut __seq: __A,
                            ) -> _serde::__private228::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::SeqAccess<'de>,
                            {
                                let __field0 = match _serde::de::SeqAccess::next_element::<
                                    bool,
                                >(&mut __seq)? {
                                    _serde::__private228::Some(__value) => __value,
                                    _serde::__private228::None => {
                                        return _serde::__private228::Err(
                                            _serde::de::Error::invalid_length(
                                                0usize,
                                                &"struct variant Authentication::Account with 3 elements",
                                            ),
                                        );
                                    }
                                };
                                let __field1 = match _serde::de::SeqAccess::next_element::<
                                    u32,
                                >(&mut __seq)? {
                                    _serde::__private228::Some(__value) => __value,
                                    _serde::__private228::None => {
                                        return _serde::__private228::Err(
                                            _serde::de::Error::invalid_length(
                                                1usize,
                                                &"struct variant Authentication::Account with 3 elements",
                                            ),
                                        );
                                    }
                                };
                                let __field2 = match _serde::de::SeqAccess::next_element::<
                                    u32,
                                >(&mut __seq)? {
                                    _serde::__private228::Some(__value) => __value,
                                    _serde::__private228::None => {
                                        return _serde::__private228::Err(
                                            _serde::de::Error::invalid_length(
                                                2usize,
                                                &"struct variant Authentication::Account with 3 elements",
                                            ),
                                        );
                                    }
                                };
                                _serde::__private228::Ok(Authentication::Account {
                                    registration_allowed: __field0,
                                    max_memory: __field1,
                                    time_cost: __field2,
                                })
                            }
                            #[inline]
                            fn visit_map<__A>(
                                self,
                                mut __map: __A,
                            ) -> _serde::__private228::Result<Self::Value, __A::Error>
                            where
                                __A: _serde::de::MapAccess<'de>,
                            {
                                let mut __field0: _serde::__private228::Option<bool> = _serde::__private228::None;
                                let mut __field1: _serde::__private228::Option<u32> = _serde::__private228::None;
                                let mut __field2: _serde::__private228::Option<u32> = _serde::__private228::None;
                                while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                    __Field,
                                >(&mut __map)? {
                                    match __key {
                                        __Field::__field0 => {
                                            if _serde::__private228::Option::is_some(&__field0) {
                                                return _serde::__private228::Err(
                                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                                        "registration_allowed",
                                                    ),
                                                );
                                            }
                                            __field0 = _serde::__private228::Some(
                                                _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                            );
                                        }
                                        __Field::__field1 => {
                                            if _serde::__private228::Option::is_some(&__field1) {
                                                return _serde::__private228::Err(
                                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                                        "max_memory",
                                                    ),
                                                );
                                            }
                                            __field1 = _serde::__private228::Some(
                                                _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                            );
                                        }
                                        __Field::__field2 => {
                                            if _serde::__private228::Option::is_some(&__field2) {
                                                return _serde::__private228::Err(
                                                    <__A::Error as _serde::de::Error>::duplicate_field(
                                                        "time_cost",
                                                    ),
                                                );
                                            }
                                            __field2 = _serde::__private228::Some(
                                                _serde::de::MapAccess::next_value::<u32>(&mut __map)?,
                                            );
                                        }
                                        _ => {
                                            let _ = _serde::de::MapAccess::next_value::<
                                                _serde::de::IgnoredAny,
                                            >(&mut __map)?;
                                        }
                                    }
                                }
                                let __field0 = match __field0 {
                                    _serde::__private228::Some(__field0) => __field0,
                                    _serde::__private228::None => {
                                        _serde::__private228::de::missing_field(
                                            "registration_allowed",
                                        )?
                                    }
                                };
                                let __field1 = match __field1 {
                                    _serde::__private228::Some(__field1) => __field1,
                                    _serde::__private228::None => {
                                        _serde::__private228::de::missing_field("max_memory")?
                                    }
                                };
                                let __field2 = match __field2 {
                                    _serde::__private228::Some(__field2) => __field2,
                                    _serde::__private228::None => {
                                        _serde::__private228::de::missing_field("time_cost")?
                                    }
                                };
                                _serde::__private228::Ok(Authentication::Account {
                                    registration_allowed: __field0,
                                    max_memory: __field1,
                                    time_cost: __field2,
                                })
                            }
                        }
                        #[doc(hidden)]
                        const FIELDS: &'static [&'static str] = &[
                            "registration_allowed",
                            "max_memory",
                            "time_cost",
                        ];
                        _serde::Deserializer::deserialize_any(
                            __deserializer,
                            __Visitor {
                                marker: _serde::__private228::PhantomData::<Authentication>,
                                lifetime: _serde::__private228::PhantomData,
                            },
                        )
                    }
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Authentication {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Authentication {
        #[inline]
        fn eq(&self, other: &Authentication) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (
                        Authentication::Account {
                            registration_allowed: __self_0,
                            max_memory: __self_1,
                            time_cost: __self_2,
                        },
                        Authentication::Account {
                            registration_allowed: __arg1_0,
                            max_memory: __arg1_1,
                            time_cost: __arg1_2,
                        },
                    ) => {
                        __self_0 == __arg1_0 && __self_1 == __arg1_1
                            && __self_2 == __arg1_2
                    }
                    _ => true,
                }
        }
    }
    impl ::zeroize::Zeroize for Authentication {
        fn zeroize(&mut self) {
            match self {
                #[allow(unused_variables)]
                Authentication::OpenToAll => {}
                #[allow(unused_variables)]
                Authentication::Account {
                    registration_allowed,
                    max_memory,
                    time_cost,
                } => {
                    registration_allowed.zeroize();
                    max_memory.zeroize();
                    time_cost.zeroize()
                }
                _ => {}
            }
        }
    }
    impl Drop for Authentication {
        fn drop(&mut self) {
            use ::zeroize::__internal::AssertZeroize;
            use ::zeroize::__internal::AssertZeroizeOnDrop;
            match self {
                #[allow(unused_variables)]
                Authentication::OpenToAll => {}
                #[allow(unused_variables)]
                Authentication::Account {
                    registration_allowed,
                    max_memory,
                    time_cost,
                } => {
                    registration_allowed.zeroize_or_on_drop();
                    max_memory.zeroize_or_on_drop();
                    time_cost.zeroize_or_on_drop()
                }
                _ => {}
            }
        }
    }
    #[doc(hidden)]
    impl ::zeroize::ZeroizeOnDrop for Authentication {}
    impl Config {
        pub async fn new() -> Returns<Self> {
            let val = fs::read_to_string("./config.json").await?;
            let out = from_str::<Self>(&val)?;
            if out.version != VERSION {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "❌ Database Config version mismatch:\n         Expected version {1}, found {0}.\n         Please migrate your configuration file to match the current schema.",
                            out.version,
                            VERSION,
                        ),
                    );
                };
            }
            Ok(out)
        }
        pub async fn new_or_default() -> Self {
            Self::new().await.unwrap_or_default()
        }
        pub async fn save_config(&self) -> Returns<()> {
            fs::write("./config.json", to_string_pretty(&self)?).await?;
            Ok(())
        }
    }
    impl Default for Config {
        fn default() -> Self {
            Self {
                version: VERSION,
                database: DatabaseConfig::default(),
                binds: def_bind(),
                admin_pass_hash: None,
                llama: LlamaConfiguration::default(),
                authentication: Authentication::Account {
                    registration_allowed: true,
                    max_memory: 64,
                    time_cost: 5,
                },
            }
        }
    }
}
use chalk_rs::Chalk;
static GLOBAL: std::alloc::System = std::alloc::System;
const _: () = {
    #[rustc_std_internal_symbol]
    unsafe fn __rust_alloc(size: usize, align: usize) -> *mut u8 {
        ::core::alloc::GlobalAlloc::alloc(
            &GLOBAL,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
        )
    }
    #[rustc_std_internal_symbol]
    unsafe fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize) -> () {
        ::core::alloc::GlobalAlloc::dealloc(
            &GLOBAL,
            ptr,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
        )
    }
    #[rustc_std_internal_symbol]
    unsafe fn __rust_realloc(
        ptr: *mut u8,
        size: usize,
        align: usize,
        new_size: usize,
    ) -> *mut u8 {
        ::core::alloc::GlobalAlloc::realloc(
            &GLOBAL,
            ptr,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
            new_size,
        )
    }
    #[rustc_std_internal_symbol]
    unsafe fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8 {
        ::core::alloc::GlobalAlloc::alloc_zeroed(
            &GLOBAL,
            ::core::alloc::Layout::from_size_align_unchecked(size, align),
        )
    }
};
fn main() {
    panic::set_hook(
        Box::new(|x| {
            let mut chalk = Chalk::new();
            if let Some(x) = x.payload_as_str() {
                {
                    ::std::io::_print(format_args!("\n"));
                };
                chalk.red().println(&"----------------");
                chalk.red().underline().println(&"An Critical Error has occured");
                chalk.reset_style();
                chalk.yellow().println(&"The server was unable to achnowledge");
                chalk.yellow().println(&"and handle the error promptly without");
                chalk.yellow().println(&"resorting to server shutdown");
                {
                    ::std::io::_print(format_args!("\n"));
                };
                {
                    ::std::io::_print(format_args!("{0}\n", x));
                };
                {
                    ::std::io::_print(format_args!("\n"));
                };
                chalk.red().println(&"----------------");
            } else {
                {
                    ::std::io::_print(format_args!("ERR: Unknown\n"));
                };
            }
            process::exit(1);
        }),
    );
    let mut args = args();
    _ = args.next();
    let mut config_ui = false;
    args.for_each(|x| {
        if &x == "config" {
            config_ui = true;
        } else {
            {
                ::core::panicking::panic_fmt(format_args!("Unknown arg: {0:?}", x));
            };
        }
    });
    if config_ui {
        ui::ui();
    } else {
        log::setup();
        server::main().unwrap();
    }
}
