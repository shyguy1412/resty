#![feature(prelude_import)]
//! This example is implements a rest api as defined in https://petstore.swagger.io/v2/swagger.json
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process::ExitCode, sync::LazyLock,
};
use resty::{Router, TcpScocket};
mod models {
    mod api_response {
        use resty::Schema;
        use serde::{Deserialize, Serialize};
        pub struct ApiResponse {
            code: i32,
            ty: String,
            message: String,
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
            impl<'de> _serde::Deserialize<'de> for ApiResponse {
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
                                "code" => _serde::__private228::Ok(__Field::__field0),
                                "ty" => _serde::__private228::Ok(__Field::__field1),
                                "message" => _serde::__private228::Ok(__Field::__field2),
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
                                b"code" => _serde::__private228::Ok(__Field::__field0),
                                b"ty" => _serde::__private228::Ok(__Field::__field1),
                                b"message" => _serde::__private228::Ok(__Field::__field2),
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
                        marker: _serde::__private228::PhantomData<ApiResponse>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = ApiResponse;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct ApiResponse",
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
                                i32,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct ApiResponse with 3 elements",
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
                                            &"struct ApiResponse with 3 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct ApiResponse with 3 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(ApiResponse {
                                code: __field0,
                                ty: __field1,
                                message: __field2,
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
                            let mut __field0: _serde::__private228::Option<i32> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field2: _serde::__private228::Option<String> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("code"),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<i32>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("ty"),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private228::Option::is_some(&__field2) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "message",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private228::Some(
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
                                    _serde::__private228::de::missing_field("code")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("ty")?
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private228::Some(__field2) => __field2,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("message")?
                                }
                            };
                            _serde::__private228::Ok(ApiResponse {
                                code: __field0,
                                ty: __field1,
                                message: __field2,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["code", "ty", "message"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "ApiResponse",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<ApiResponse>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for ApiResponse {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "ApiResponse",
                        false as usize + 1 + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "code",
                        &self.code,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "ty",
                        &self.ty,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "message",
                        &self.message,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ::resty::__private::Schema for ApiResponse {}
    }
    use std::{net::TcpStream, pin::Pin};
    pub use api_response::*;
    mod category {
        use resty::Schema;
        use serde::{Deserialize, Serialize};
        pub struct Category {
            #[schema(Example(1))]
            id: i64,
            #[schema(Example("Dogs"))]
            name: String,
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
            impl<'de> _serde::Deserialize<'de> for Category {
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
                                "id" => _serde::__private228::Ok(__Field::__field0),
                                "name" => _serde::__private228::Ok(__Field::__field1),
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
                                b"id" => _serde::__private228::Ok(__Field::__field0),
                                b"name" => _serde::__private228::Ok(__Field::__field1),
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
                        marker: _serde::__private228::PhantomData<Category>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Category;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct Category",
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
                                i64,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Category with 2 elements",
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
                                            &"struct Category with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(Category {
                                id: __field0,
                                name: __field1,
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
                            let mut __field0: _serde::__private228::Option<i64> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<i64>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("name"),
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
                                    _serde::__private228::de::missing_field("id")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("name")?
                                }
                            };
                            _serde::__private228::Ok(Category {
                                id: __field0,
                                name: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["id", "name"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Category",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<Category>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for Category {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "Category",
                        false as usize + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "id",
                        &self.id,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "name",
                        &self.name,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ::resty::__private::Schema for Category {}
    }
    pub use category::*;
    mod order {
        use resty::Schema;
        use serde::{Deserialize, Serialize};
        #[schema(Description("Some Order"))]
        pub struct Order {
            #[schema(Example(10))]
            id: i64,
            #[schema(Example("doggie"))]
            pet_id: String,
            #[schema(Example(7))]
            quantity: i32,
            #[schema(Format("date-time"))]
            ship_date: String,
            #[schema(Ref(OrderStatus), Description("Order Status"), Example("approved"))]
            status: Status,
            complete: bool,
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
            impl<'de> _serde::Deserialize<'de> for Order {
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
                                "id" => _serde::__private228::Ok(__Field::__field0),
                                "pet_id" => _serde::__private228::Ok(__Field::__field1),
                                "quantity" => _serde::__private228::Ok(__Field::__field2),
                                "ship_date" => _serde::__private228::Ok(__Field::__field3),
                                "status" => _serde::__private228::Ok(__Field::__field4),
                                "complete" => _serde::__private228::Ok(__Field::__field5),
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
                                b"id" => _serde::__private228::Ok(__Field::__field0),
                                b"pet_id" => _serde::__private228::Ok(__Field::__field1),
                                b"quantity" => _serde::__private228::Ok(__Field::__field2),
                                b"ship_date" => _serde::__private228::Ok(__Field::__field3),
                                b"status" => _serde::__private228::Ok(__Field::__field4),
                                b"complete" => _serde::__private228::Ok(__Field::__field5),
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
                        marker: _serde::__private228::PhantomData<Order>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Order;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct Order",
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
                                i64,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Order with 6 elements",
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
                                            &"struct Order with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                i32,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct Order with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field3 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct Order with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field4 = match _serde::de::SeqAccess::next_element::<
                                Status,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct Order with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field5 = match _serde::de::SeqAccess::next_element::<
                                bool,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            5usize,
                                            &"struct Order with 6 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(Order {
                                id: __field0,
                                pet_id: __field1,
                                quantity: __field2,
                                ship_date: __field3,
                                status: __field4,
                                complete: __field5,
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
                            let mut __field0: _serde::__private228::Option<i64> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field2: _serde::__private228::Option<i32> = _serde::__private228::None;
                            let mut __field3: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field4: _serde::__private228::Option<Status> = _serde::__private228::None;
                            let mut __field5: _serde::__private228::Option<bool> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<i64>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("pet_id"),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private228::Option::is_some(&__field2) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "quantity",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<i32>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::__private228::Option::is_some(&__field3) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "ship_date",
                                                ),
                                            );
                                        }
                                        __field3 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field4 => {
                                        if _serde::__private228::Option::is_some(&__field4) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("status"),
                                            );
                                        }
                                        __field4 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<Status>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field5 => {
                                        if _serde::__private228::Option::is_some(&__field5) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "complete",
                                                ),
                                            );
                                        }
                                        __field5 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
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
                                    _serde::__private228::de::missing_field("id")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("pet_id")?
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private228::Some(__field2) => __field2,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("quantity")?
                                }
                            };
                            let __field3 = match __field3 {
                                _serde::__private228::Some(__field3) => __field3,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("ship_date")?
                                }
                            };
                            let __field4 = match __field4 {
                                _serde::__private228::Some(__field4) => __field4,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("status")?
                                }
                            };
                            let __field5 = match __field5 {
                                _serde::__private228::Some(__field5) => __field5,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("complete")?
                                }
                            };
                            _serde::__private228::Ok(Order {
                                id: __field0,
                                pet_id: __field1,
                                quantity: __field2,
                                ship_date: __field3,
                                status: __field4,
                                complete: __field5,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "id",
                        "pet_id",
                        "quantity",
                        "ship_date",
                        "status",
                        "complete",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Order",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<Order>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for Order {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "Order",
                        false as usize + 1 + 1 + 1 + 1 + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "id",
                        &self.id,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "pet_id",
                        &self.pet_id,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "quantity",
                        &self.quantity,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "ship_date",
                        &self.ship_date,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "status",
                        &self.status,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "complete",
                        &self.complete,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ::resty::__private::Schema for Order {}
        #[schema(Name(OrderStatus), Type("string"))]
        enum Status {
            #[schema(Repr(placed))]
            OrderPlaced,
            #[schema(Example)]
            Approved,
            Delivered,
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
            impl<'de> _serde::Deserialize<'de> for Status {
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
                                "OrderPlaced" => _serde::__private228::Ok(__Field::__field0),
                                "Approved" => _serde::__private228::Ok(__Field::__field1),
                                "Delivered" => _serde::__private228::Ok(__Field::__field2),
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
                                b"OrderPlaced" => {
                                    _serde::__private228::Ok(__Field::__field0)
                                }
                                b"Approved" => _serde::__private228::Ok(__Field::__field1),
                                b"Delivered" => _serde::__private228::Ok(__Field::__field2),
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
                    struct __Visitor<'de> {
                        marker: _serde::__private228::PhantomData<Status>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Status;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "enum Status",
                            )
                        }
                        fn visit_enum<__A>(
                            self,
                            __data: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                        {
                            match _serde::de::EnumAccess::variant(__data)? {
                                (__Field::__field0, __variant) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private228::Ok(Status::OrderPlaced)
                                }
                                (__Field::__field1, __variant) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private228::Ok(Status::Approved)
                                }
                                (__Field::__field2, __variant) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private228::Ok(Status::Delivered)
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &[
                        "OrderPlaced",
                        "Approved",
                        "Delivered",
                    ];
                    _serde::Deserializer::deserialize_enum(
                        __deserializer,
                        "Status",
                        VARIANTS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<Status>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for Status {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        Status::OrderPlaced => {
                            _serde::Serializer::serialize_unit_variant(
                                __serializer,
                                "Status",
                                0u32,
                                "OrderPlaced",
                            )
                        }
                        Status::Approved => {
                            _serde::Serializer::serialize_unit_variant(
                                __serializer,
                                "Status",
                                1u32,
                                "Approved",
                            )
                        }
                        Status::Delivered => {
                            _serde::Serializer::serialize_unit_variant(
                                __serializer,
                                "Status",
                                2u32,
                                "Delivered",
                            )
                        }
                    }
                }
            }
        };
        impl ::resty::__private::Schema for Status {}
    }
    pub use order::*;
    mod pet {
        use resty::Schema;
        use serde::{Deserialize, Serialize};
        pub struct Pet {
            #[schema(Example(10), Required)]
            id: Option<i64>,
            #[schema(Example("doggie"))]
            name: String,
            category: Option<super::Category>,
            photo_urls: Option<Vec<String>>,
            tags: Option<super::Tag>,
            #[schema(Ref(PetStatus))]
            status: Option<Status>,
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
            impl<'de> _serde::Deserialize<'de> for Pet {
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
                                "id" => _serde::__private228::Ok(__Field::__field0),
                                "name" => _serde::__private228::Ok(__Field::__field1),
                                "category" => _serde::__private228::Ok(__Field::__field2),
                                "photo_urls" => _serde::__private228::Ok(__Field::__field3),
                                "tags" => _serde::__private228::Ok(__Field::__field4),
                                "status" => _serde::__private228::Ok(__Field::__field5),
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
                                b"id" => _serde::__private228::Ok(__Field::__field0),
                                b"name" => _serde::__private228::Ok(__Field::__field1),
                                b"category" => _serde::__private228::Ok(__Field::__field2),
                                b"photo_urls" => _serde::__private228::Ok(__Field::__field3),
                                b"tags" => _serde::__private228::Ok(__Field::__field4),
                                b"status" => _serde::__private228::Ok(__Field::__field5),
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
                        marker: _serde::__private228::PhantomData<Pet>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Pet;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct Pet",
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
                                Option<i64>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Pet with 6 elements",
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
                                            &"struct Pet with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                Option<super::Category>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct Pet with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field3 = match _serde::de::SeqAccess::next_element::<
                                Option<Vec<String>>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct Pet with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field4 = match _serde::de::SeqAccess::next_element::<
                                Option<super::Tag>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct Pet with 6 elements",
                                        ),
                                    );
                                }
                            };
                            let __field5 = match _serde::de::SeqAccess::next_element::<
                                Option<Status>,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            5usize,
                                            &"struct Pet with 6 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(Pet {
                                id: __field0,
                                name: __field1,
                                category: __field2,
                                photo_urls: __field3,
                                tags: __field4,
                                status: __field5,
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
                                Option<i64>,
                            > = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field2: _serde::__private228::Option<
                                Option<super::Category>,
                            > = _serde::__private228::None;
                            let mut __field3: _serde::__private228::Option<
                                Option<Vec<String>>,
                            > = _serde::__private228::None;
                            let mut __field4: _serde::__private228::Option<
                                Option<super::Tag>,
                            > = _serde::__private228::None;
                            let mut __field5: _serde::__private228::Option<
                                Option<Status>,
                            > = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<i64>,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private228::Option::is_some(&__field2) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "category",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<super::Category>,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::__private228::Option::is_some(&__field3) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "photo_urls",
                                                ),
                                            );
                                        }
                                        __field3 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<Vec<String>>,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field4 => {
                                        if _serde::__private228::Option::is_some(&__field4) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("tags"),
                                            );
                                        }
                                        __field4 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<super::Tag>,
                                            >(&mut __map)?,
                                        );
                                    }
                                    __Field::__field5 => {
                                        if _serde::__private228::Option::is_some(&__field5) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("status"),
                                            );
                                        }
                                        __field5 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<
                                                Option<Status>,
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
                                    _serde::__private228::de::missing_field("id")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("name")?
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private228::Some(__field2) => __field2,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("category")?
                                }
                            };
                            let __field3 = match __field3 {
                                _serde::__private228::Some(__field3) => __field3,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("photo_urls")?
                                }
                            };
                            let __field4 = match __field4 {
                                _serde::__private228::Some(__field4) => __field4,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("tags")?
                                }
                            };
                            let __field5 = match __field5 {
                                _serde::__private228::Some(__field5) => __field5,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("status")?
                                }
                            };
                            _serde::__private228::Ok(Pet {
                                id: __field0,
                                name: __field1,
                                category: __field2,
                                photo_urls: __field3,
                                tags: __field4,
                                status: __field5,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "id",
                        "name",
                        "category",
                        "photo_urls",
                        "tags",
                        "status",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Pet",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<Pet>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for Pet {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "Pet",
                        false as usize + 1 + 1 + 1 + 1 + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "id",
                        &self.id,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "name",
                        &self.name,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "category",
                        &self.category,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "photo_urls",
                        &self.photo_urls,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "tags",
                        &self.tags,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "status",
                        &self.status,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ::resty::__private::Schema for Pet {}
        #[schema(Name(PetStatus))]
        enum Status {
            #[schema(Example)]
            Available,
            Pending,
            Sold,
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
            impl<'de> _serde::Deserialize<'de> for Status {
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
                                "Available" => _serde::__private228::Ok(__Field::__field0),
                                "Pending" => _serde::__private228::Ok(__Field::__field1),
                                "Sold" => _serde::__private228::Ok(__Field::__field2),
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
                                b"Available" => _serde::__private228::Ok(__Field::__field0),
                                b"Pending" => _serde::__private228::Ok(__Field::__field1),
                                b"Sold" => _serde::__private228::Ok(__Field::__field2),
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
                    struct __Visitor<'de> {
                        marker: _serde::__private228::PhantomData<Status>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Status;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "enum Status",
                            )
                        }
                        fn visit_enum<__A>(
                            self,
                            __data: __A,
                        ) -> _serde::__private228::Result<Self::Value, __A::Error>
                        where
                            __A: _serde::de::EnumAccess<'de>,
                        {
                            match _serde::de::EnumAccess::variant(__data)? {
                                (__Field::__field0, __variant) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private228::Ok(Status::Available)
                                }
                                (__Field::__field1, __variant) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private228::Ok(Status::Pending)
                                }
                                (__Field::__field2, __variant) => {
                                    _serde::de::VariantAccess::unit_variant(__variant)?;
                                    _serde::__private228::Ok(Status::Sold)
                                }
                            }
                        }
                    }
                    #[doc(hidden)]
                    const VARIANTS: &'static [&'static str] = &[
                        "Available",
                        "Pending",
                        "Sold",
                    ];
                    _serde::Deserializer::deserialize_enum(
                        __deserializer,
                        "Status",
                        VARIANTS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<Status>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for Status {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    match *self {
                        Status::Available => {
                            _serde::Serializer::serialize_unit_variant(
                                __serializer,
                                "Status",
                                0u32,
                                "Available",
                            )
                        }
                        Status::Pending => {
                            _serde::Serializer::serialize_unit_variant(
                                __serializer,
                                "Status",
                                1u32,
                                "Pending",
                            )
                        }
                        Status::Sold => {
                            _serde::Serializer::serialize_unit_variant(
                                __serializer,
                                "Status",
                                2u32,
                                "Sold",
                            )
                        }
                    }
                }
            }
        };
        impl ::resty::__private::Schema for Status {}
    }
    pub use pet::*;
    mod tag {
        use resty::Schema;
        use serde::{Deserialize, Serialize};
        pub struct Tag {
            id: i64,
            name: String,
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
            impl<'de> _serde::Deserialize<'de> for Tag {
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
                                "id" => _serde::__private228::Ok(__Field::__field0),
                                "name" => _serde::__private228::Ok(__Field::__field1),
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
                                b"id" => _serde::__private228::Ok(__Field::__field0),
                                b"name" => _serde::__private228::Ok(__Field::__field1),
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
                        marker: _serde::__private228::PhantomData<Tag>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = Tag;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct Tag",
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
                                i64,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct Tag with 2 elements",
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
                                            &"struct Tag with 2 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(Tag {
                                id: __field0,
                                name: __field1,
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
                            let mut __field0: _serde::__private228::Option<i64> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<i64>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("name"),
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
                                    _serde::__private228::de::missing_field("id")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("name")?
                                }
                            };
                            _serde::__private228::Ok(Tag {
                                id: __field0,
                                name: __field1,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &["id", "name"];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "Tag",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<Tag>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for Tag {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "Tag",
                        false as usize + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "id",
                        &self.id,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "name",
                        &self.name,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ::resty::__private::Schema for Tag {}
    }
    use resty::DeserializeStream;
    pub use tag::*;
    mod user {
        use resty::Schema;
        use serde::{Deserialize, Serialize};
        pub struct User {
            #[schema(Example(10))]
            id: i64,
            #[schema(Example("theUser"))]
            username: String,
            #[schema(Example("John"))]
            first_name: String,
            #[schema(Example("James"))]
            last_name: String,
            #[schema(Example("john@email.com"))]
            email: String,
            #[schema(Example("12345"))]
            password: String,
            #[schema(Example("12345"))]
            phone: String,
            #[schema(Example(1), Description("User Status"))]
            user_status: i32,
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
            impl<'de> _serde::Deserialize<'de> for User {
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
                        __field6,
                        __field7,
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
                                6u64 => _serde::__private228::Ok(__Field::__field6),
                                7u64 => _serde::__private228::Ok(__Field::__field7),
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
                                "id" => _serde::__private228::Ok(__Field::__field0),
                                "username" => _serde::__private228::Ok(__Field::__field1),
                                "first_name" => _serde::__private228::Ok(__Field::__field2),
                                "last_name" => _serde::__private228::Ok(__Field::__field3),
                                "email" => _serde::__private228::Ok(__Field::__field4),
                                "password" => _serde::__private228::Ok(__Field::__field5),
                                "phone" => _serde::__private228::Ok(__Field::__field6),
                                "user_status" => _serde::__private228::Ok(__Field::__field7),
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
                                b"id" => _serde::__private228::Ok(__Field::__field0),
                                b"username" => _serde::__private228::Ok(__Field::__field1),
                                b"first_name" => _serde::__private228::Ok(__Field::__field2),
                                b"last_name" => _serde::__private228::Ok(__Field::__field3),
                                b"email" => _serde::__private228::Ok(__Field::__field4),
                                b"password" => _serde::__private228::Ok(__Field::__field5),
                                b"phone" => _serde::__private228::Ok(__Field::__field6),
                                b"user_status" => {
                                    _serde::__private228::Ok(__Field::__field7)
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
                        marker: _serde::__private228::PhantomData<User>,
                        lifetime: _serde::__private228::PhantomData<&'de ()>,
                    }
                    #[automatically_derived]
                    impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                        type Value = User;
                        fn expecting(
                            &self,
                            __formatter: &mut _serde::__private228::Formatter,
                        ) -> _serde::__private228::fmt::Result {
                            _serde::__private228::Formatter::write_str(
                                __formatter,
                                "struct User",
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
                                i64,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            0usize,
                                            &"struct User with 8 elements",
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
                                            &"struct User with 8 elements",
                                        ),
                                    );
                                }
                            };
                            let __field2 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            2usize,
                                            &"struct User with 8 elements",
                                        ),
                                    );
                                }
                            };
                            let __field3 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            3usize,
                                            &"struct User with 8 elements",
                                        ),
                                    );
                                }
                            };
                            let __field4 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            4usize,
                                            &"struct User with 8 elements",
                                        ),
                                    );
                                }
                            };
                            let __field5 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            5usize,
                                            &"struct User with 8 elements",
                                        ),
                                    );
                                }
                            };
                            let __field6 = match _serde::de::SeqAccess::next_element::<
                                String,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            6usize,
                                            &"struct User with 8 elements",
                                        ),
                                    );
                                }
                            };
                            let __field7 = match _serde::de::SeqAccess::next_element::<
                                i32,
                            >(&mut __seq)? {
                                _serde::__private228::Some(__value) => __value,
                                _serde::__private228::None => {
                                    return _serde::__private228::Err(
                                        _serde::de::Error::invalid_length(
                                            7usize,
                                            &"struct User with 8 elements",
                                        ),
                                    );
                                }
                            };
                            _serde::__private228::Ok(User {
                                id: __field0,
                                username: __field1,
                                first_name: __field2,
                                last_name: __field3,
                                email: __field4,
                                password: __field5,
                                phone: __field6,
                                user_status: __field7,
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
                            let mut __field0: _serde::__private228::Option<i64> = _serde::__private228::None;
                            let mut __field1: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field2: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field3: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field4: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field5: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field6: _serde::__private228::Option<String> = _serde::__private228::None;
                            let mut __field7: _serde::__private228::Option<i32> = _serde::__private228::None;
                            while let _serde::__private228::Some(__key) = _serde::de::MapAccess::next_key::<
                                __Field,
                            >(&mut __map)? {
                                match __key {
                                    __Field::__field0 => {
                                        if _serde::__private228::Option::is_some(&__field0) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                            );
                                        }
                                        __field0 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<i64>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field1 => {
                                        if _serde::__private228::Option::is_some(&__field1) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "username",
                                                ),
                                            );
                                        }
                                        __field1 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field2 => {
                                        if _serde::__private228::Option::is_some(&__field2) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "first_name",
                                                ),
                                            );
                                        }
                                        __field2 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field3 => {
                                        if _serde::__private228::Option::is_some(&__field3) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "last_name",
                                                ),
                                            );
                                        }
                                        __field3 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field4 => {
                                        if _serde::__private228::Option::is_some(&__field4) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("email"),
                                            );
                                        }
                                        __field4 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field5 => {
                                        if _serde::__private228::Option::is_some(&__field5) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "password",
                                                ),
                                            );
                                        }
                                        __field5 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field6 => {
                                        if _serde::__private228::Option::is_some(&__field6) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field("phone"),
                                            );
                                        }
                                        __field6 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                        );
                                    }
                                    __Field::__field7 => {
                                        if _serde::__private228::Option::is_some(&__field7) {
                                            return _serde::__private228::Err(
                                                <__A::Error as _serde::de::Error>::duplicate_field(
                                                    "user_status",
                                                ),
                                            );
                                        }
                                        __field7 = _serde::__private228::Some(
                                            _serde::de::MapAccess::next_value::<i32>(&mut __map)?,
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
                                    _serde::__private228::de::missing_field("id")?
                                }
                            };
                            let __field1 = match __field1 {
                                _serde::__private228::Some(__field1) => __field1,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("username")?
                                }
                            };
                            let __field2 = match __field2 {
                                _serde::__private228::Some(__field2) => __field2,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("first_name")?
                                }
                            };
                            let __field3 = match __field3 {
                                _serde::__private228::Some(__field3) => __field3,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("last_name")?
                                }
                            };
                            let __field4 = match __field4 {
                                _serde::__private228::Some(__field4) => __field4,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("email")?
                                }
                            };
                            let __field5 = match __field5 {
                                _serde::__private228::Some(__field5) => __field5,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("password")?
                                }
                            };
                            let __field6 = match __field6 {
                                _serde::__private228::Some(__field6) => __field6,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("phone")?
                                }
                            };
                            let __field7 = match __field7 {
                                _serde::__private228::Some(__field7) => __field7,
                                _serde::__private228::None => {
                                    _serde::__private228::de::missing_field("user_status")?
                                }
                            };
                            _serde::__private228::Ok(User {
                                id: __field0,
                                username: __field1,
                                first_name: __field2,
                                last_name: __field3,
                                email: __field4,
                                password: __field5,
                                phone: __field6,
                                user_status: __field7,
                            })
                        }
                    }
                    #[doc(hidden)]
                    const FIELDS: &'static [&'static str] = &[
                        "id",
                        "username",
                        "first_name",
                        "last_name",
                        "email",
                        "password",
                        "phone",
                        "user_status",
                    ];
                    _serde::Deserializer::deserialize_struct(
                        __deserializer,
                        "User",
                        FIELDS,
                        __Visitor {
                            marker: _serde::__private228::PhantomData::<User>,
                            lifetime: _serde::__private228::PhantomData,
                        },
                    )
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
            impl _serde::Serialize for User {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private228::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "User",
                        false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "id",
                        &self.id,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "username",
                        &self.username,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "first_name",
                        &self.first_name,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "last_name",
                        &self.last_name,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "email",
                        &self.email,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "password",
                        &self.password,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "phone",
                        &self.phone,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "user_status",
                        &self.user_status,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ::resty::__private::Schema for User {}
    }
    pub use user::*;
    pub struct Json<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
    impl<T> resty::Deserialize for Json<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        async fn deserialize<'a, 'b: 'a>(
            data: &'a mut DeserializeStream<'b>,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Json(serde_json::from_reader(Into::<resty::SyncReader>::into(data))?))
        }
    }
    impl<T> resty::Serialize for Json<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            Ok(serde_json::to_vec(&self.0)?)
        }
    }
    pub struct XML<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
    impl<T> resty::Deserialize for XML<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        async fn deserialize<'a, 'b: 'a>(
            data: &'a mut DeserializeStream<'b>,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(XML(serde_xml_rs::from_reader(Into::<resty::SyncReader>::into(data))?))
        }
    }
    impl<T> resty::Serialize for XML<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut vec = Vec::new();
            serde_xml_rs::to_writer(&mut vec, &self.0)?;
            Ok(vec)
        }
    }
}
#[allow(non_snake_case)]
#[path = "./routes"]
mod ROUTER {
    use ::resty::__private::*;
    #[doc(hidden)]
    pub static ROUTER: linkme::DistributedSlice<[::resty::RouteSlice]> = {
        unsafe extern "Rust" {
            #[link_name = "__start_linkme_ROUTER"]
            static LINKME_START: [<[::resty::RouteSlice] as linkme::__private36::Slice>::Element; 0];
            #[link_name = "__stop_linkme_ROUTER"]
            static LINKME_STOP: [<[::resty::RouteSlice] as linkme::__private36::Slice>::Element; 0];
            #[link_name = "__start_linkm2_ROUTER"]
            static DUPCHECK_START: ();
            #[link_name = "__stop_linkm2_ROUTER"]
            static DUPCHECK_STOP: ();
        }
        #[used]
        #[unsafe(link_section = "linkme_ROUTER")]
        static mut LINKME_PLEASE: [<[::resty::RouteSlice] as linkme::__private36::Slice>::Element; 0] = [];
        #[used]
        #[unsafe(link_section = "linkm2_ROUTER")]
        static DUPCHECK: linkme::__private36::isize = 1;
        unsafe {
            linkme::DistributedSlice::private_new(
                "ROUTER",
                (&raw const LINKME_START)
                    .cast::<
                        <[::resty::RouteSlice] as linkme::__private36::Slice>::Element,
                    >(),
                (&raw const LINKME_STOP)
                    .cast::<
                        <[::resty::RouteSlice] as linkme::__private36::Slice>::Element,
                    >(),
                (&raw const DUPCHECK_START).cast::<linkme::__private36::isize>(),
                (&raw const DUPCHECK_STOP).cast::<linkme::__private36::isize>(),
            )
        }
    };
    #[doc(hidden)]
    pub use _linkme_macro_ROUTER as ROUTER;
    #[path = "user/mod.rs"]
    mod __endpoint_user_mod {}
    #[path = "user/[username].rs"]
    mod __endpoint_user_username_ {}
    #[path = "user/createWithList.rs"]
    mod __endpoint_user_createWithList {}
    #[path = "user/createWithArray.rs"]
    mod __endpoint_user_createWithArray {}
    #[path = "user/logout.rs"]
    mod __endpoint_user_logout {}
    #[path = "user/login.rs"]
    mod __endpoint_user_login {}
    #[path = "pet/mod.rs"]
    mod __endpoint_pet_mod {
        use resty::{Request, Response, endpoint};
        use crate::models::{Json, Pet, XML};
        pub fn put_pet<'a, 'data, '__fn_borrow>(
            req: &'__fn_borrow mut ::resty::Request<'a, 'data>,
            res: &'__fn_borrow mut ::resty::Response<'a>,
        ) -> ::resty::EndpointTask<'__fn_borrow> {
            async fn put_pet<'a, 'b>(
                req: &mut Request<'a, 'b>,
                res: &mut Response<'a>,
            ) -> resty::Result {
                let Json(body): Json<Pet> = req.body().await?;
                Ok(())
            }
            const STATIC_HEADERS: &[(&str, &str)] = &[];
            Box::pin(async move {
                put_pet(req, res).await?;
                Ok(())
            })
        }
        use ::resty::__private::*;
        #[used]
        #[unsafe(link_section = "linkme_ROUTER")]
        static __put_pet_route: ::resty::RouteSlice = {
            #[allow(clippy::no_effect_underscore_binding)]
            unsafe fn __typecheck(_: linkme::__private36::Void) {
                #[allow(clippy::ref_option_ref)]
                let __new = || -> fn() -> &'static ::resty::RouteSlice {
                    || &__put_pet_route
                };
                unsafe {
                    linkme::DistributedSlice::private_typecheck(super::ROUTER, __new());
                }
            }
            (
                &["pet"],
                ::resty::Handler(
                    &put_pet,
                    {
                        use ::resty::HttpMethod::*;
                        PUT as u16
                    },
                ),
            )
        };
        pub fn post_pet<'a, 'data, '__fn_borrow>(
            req: &'__fn_borrow mut ::resty::Request<'a, 'data>,
            res: &'__fn_borrow mut ::resty::Response<'a>,
        ) -> ::resty::EndpointTask<'__fn_borrow> {
            async fn post_pet<'a, 'b>(
                req: &mut Request<'a, 'b>,
                res: &mut Response<'a>,
            ) -> resty::Result {
                res.ok(&"Ok").await?;
                Ok(())
            }
            const STATIC_HEADERS: &[(&str, &str)] = &[];
            Box::pin(async move {
                post_pet(req, res).await?;
                Ok(())
            })
        }
        use ::resty::__private::*;
        #[used]
        #[unsafe(link_section = "linkme_ROUTER")]
        static __post_pet_route: ::resty::RouteSlice = {
            #[allow(clippy::no_effect_underscore_binding)]
            unsafe fn __typecheck(_: linkme::__private36::Void) {
                #[allow(clippy::ref_option_ref)]
                let __new = || -> fn() -> &'static ::resty::RouteSlice {
                    || &__post_pet_route
                };
                unsafe {
                    linkme::DistributedSlice::private_typecheck(super::ROUTER, __new());
                }
            }
            (
                &["pet"],
                ::resty::Handler(
                    &post_pet,
                    {
                        use ::resty::HttpMethod::*;
                        POST as u16
                    },
                ),
            )
        };
    }
    #[path = "pet/[pet_id]/uploadImage.rs"]
    mod __endpoint_pet_pet_id__uploadImage {}
    #[path = "pet/[pet_id]/mod.rs"]
    mod __endpoint_pet_pet_id__mod {}
    #[path = "store/order/mod.rs"]
    mod __endpoint_store_order_mod {}
    #[path = "store/order/[order_id].rs"]
    mod __endpoint_store_order_order_id_ {}
    #[path = "store/inventory.rs"]
    mod __endpoint_store_inventory {}
}
static ROUTER: LazyLock<Router> = ::std::sync::LazyLock::new(|| ::resty::Router::new(
    &ROUTER::ROUTER,
));
fn main() -> ExitCode {
    {
        ::std::io::_print(format_args!("{0}\n", *ROUTER));
    };
    const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3333);
    if let Err(error) = resty::bind::<TcpScocket>(ADDR, &ROUTER) {
        {
            ::std::io::_print(format_args!("{0:?}\n", error));
        };
        return ExitCode::FAILURE;
    }
    {
        ::std::io::_print(format_args!("Listening on port 3333\n"));
    };
    let _: Vec<_> = std::thread::available_parallelism()
        .ok()
        .map(|n| 0..n.get())
        .unwrap_or(0..1)
        .map(|_| resty::spawn_thread())
        .collect();
    std::thread::park();
    return ExitCode::SUCCESS;
}
