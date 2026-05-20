macro_rules! impl_json_methods {
    ($ty:ty, $err:ty) => {
        impl_json_methods! {
            $ty,
            $err,
            from_json(json) {
                Ok(crate::serde_json::from_slice(json.as_ref())?)
            }
        }
    };

    ($ty:ty, $err:ty, from_json($json:ident) $from_json:block) => {
        impl $ty {
            /// Deserialize from JSON.
            #[inline]
            #[allow(clippy::needless_question_mark)]
            pub fn from_json<T>($json: T) -> Result<Self, $err>
            where
                T: AsRef<[u8]>,
            {
                $from_json
            }

            /// Serialize as JSON string.
            ///
            /// This method could panic. Use `try_as_json` for error propagation.
            #[inline]
            pub fn as_json(&self) -> alloc::string::String {
                self.try_as_json().unwrap()
            }

            /// Serialize as JSON string.
            #[inline]
            #[allow(clippy::needless_question_mark)]
            pub fn try_as_json(&self) -> Result<alloc::string::String, $err> {
                Ok(serde_json::to_string(self)?)
            }

            /// Serialize as pretty JSON string.
            ///
            /// This method could panic. Use `try_as_pretty_json` for error propagation.
            #[inline]
            pub fn as_pretty_json(&self) -> alloc::string::String {
                self.try_as_pretty_json().unwrap()
            }

            /// Serialize as pretty JSON string.
            #[inline]
            #[allow(clippy::needless_question_mark)]
            pub fn try_as_pretty_json(&self) -> Result<alloc::string::String, $err> {
                Ok(serde_json::to_string_pretty(self)?)
            }
        }
    };
}

pub(crate) use impl_json_methods;
