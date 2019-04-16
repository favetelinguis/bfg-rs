/// Automatically generate From impls for types given using a small DSL like
/// macro
//TODO I need to handle errors when building up request and I dont want to return Result objects
// TryFrom TryInto should fix this or I could check what github-rs is doing right now its no error checks.
macro_rules! from {
    (
        $(@$f: ident
            $( ?> $i1: ident = $e1: tt )*
            $( => $t: ident )*
            $( -> $i2: ident = $e2: tt )*
        )*
    )=> {
            $(
                $(
                    impl <'g> From<$f<'g>> for $t<'g> {
                        fn from(f: $f<'g>) -> Self {
                            Self {
                                request: f.request,
                                client: f.client,
                                parameter: None,
                            }
                        }
                    }
                )*
                $(
                    impl <'g> From<$f<'g>> for $i1<'g> {
                        fn from(mut f: $f<'g>) -> Self {
                            // TODO this is how I should build query params not impl atm
                            Self {
                                request: f.request,
                                client: f.client,
                                parameter: None,
                            }
                        }
                    }
                )*
                $(
                    impl <'g> From<$f<'g>> for $i2<'g> {
                        // TODO put the versioning in the dsl instead of having it in a string split
                        fn from(mut f: $f<'g>) -> Self {
                            let mut split_it = $e2.split("|");
                            if let Some(path) = split_it.next() {
                                f.request.borrow_mut()
                                    .url_mut()
                                    .path_segments_mut()
                                    .unwrap()
                                    .push(path);
                            }
                            match split_it.next() {
                                Some(version) => {
                                    println!("Here1");
                                    f.request.borrow_mut()
                                        .headers_mut()
                                        .insert(HeaderName::from_static("version"),
                                                HeaderValue::from_static(version));
                                }
                                None => {
                                    println!("Here2");
                                    f.request.borrow_mut()
                                        .headers_mut()
                                        .insert(HeaderName::from_static("version"),
                                                HeaderValue::from_static("1"));

                                }
                            }
                            Self {
                                request: f.request,
                                client: f.client,
                                parameter: None,
                            }
                        }
                    }
                )*
            )*
    };
    (
        $(@$t: ident => $p: expr)*
    )=> {
            $(
                impl <'g> From<&'g IG> for $t<'g> {
                    fn from(ig: &'g IG) -> Self {
                        use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT, HeaderName, HeaderValue };
                        let mut req = Request::new($p, Url::parse("https://demo-api.ig.com/gateway/deal").unwrap());
                        let headers = req.headers_mut();
                        if let Some(token) = & ig.token {
                            let auth = String::from("Bearer ") + &token;
                            headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth).unwrap());
                        }
                        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                        headers.insert(USER_AGENT, HeaderValue::from_static("ig-rs"));
                        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                        headers.insert(HeaderName::from_static("x-ig-api-key"), ig.api_key.parse().unwrap());
                        headers.insert(HeaderName::from_static("ig-account-id"), ig.account.parse().unwrap());
                        // TODO check the enum of client if im logged in, if im not logged in return error
                        Self {
                            request: RefCell::new(req),
                            client: & ig.client,
                            parameter: None,
                        }
                    }
                }
            )*
    };
}

/// Using a small DSL like macro generate an impl for a given type
/// that creates all the functions to transition from one node type to another
macro_rules! impl_macro {
    ($(@$i: ident $(|=> $id1: ident -> $t1: ident)*|
     $(|=> $id2: ident -> $t2: ident = $e2: ident)*
     $(|?> $id3: ident -> $t3: ident = $e3: ident)*)+
    )=> (
        $(
            impl<'g> $i <'g>{
            $(
                pub fn $id1(self) -> $t1<'g> {
                    self.into()
                }
            )*$(
                pub fn $id2(mut self, $e2: &str) -> $t2<'g> {
                    // TODO not implemented
                    self.into()
                }
            )*$(
                pub fn $id3(mut self, $e3: &str) -> $t3<'g> {
                    // TODO not implemented
                    self.into()
                }
            )*
            }
        )+
    );
}

/// A variation of `impl_macro` for the client module that allows partitioning of
/// types. Create a function with a given name and return type. Used for
/// creating functions for simple conversions from one type to another, where
/// the actual conversion code is in the From implementation.
macro_rules! func_client{
    ($i: ident, $t: ty) => (
        pub fn $i(self) -> $t {
            self.into()
        }
    );
}

// TODO Why do i need the 'g here?
macro_rules! new_type {
    ($($i: ident)*) => (
        $(
        pub struct $i<'g> {
            pub(crate) request: RefCell<Request>,
            pub(crate) client: &'g Rc<Client>,
            pub(crate) parameter: Option<String>,
        }
        )*
    );
}

/// Used to generate an execute function for a terminal type in a query
/// pipeline. If passed a type it creates the impl as well as it needs
/// no extra functions.
macro_rules! exec {
    ($t: ident) => {
        impl<'a> Executor for $t<'a> {
            /// Execute the query by sending the built up request to GitHub.
            /// The value returned is either an error or the Status Code and
            /// Json after it has been deserialized. Please take a look at
            /// the GitHub documentation to see what value you should receive
            /// back for good or bad requests.
            fn execute<T>(self) -> Result<(HeaderMap, StatusCode, Option<T>)>
            where
                T: DeserializeOwned,
            {
                //TODO in original code
                //core used try_borrow_mut to make sure only a single query can be executed at time
                //do i need to think about this with client?
                let client = self.client;
                let req = self.request.into_inner();
                let mut res = client.execute(req)?;
                let header = res.headers().clone();
                let status = res.status();
                let body = res.json()?;
                // TODO should i have a Ok((header, status, None)) return if json empty
                Ok((header, status, Some(body)))
            }
        }
    };
}

/// Common imports for every file
macro_rules! imports {
    () => {
        use reqwest::{Client, Method, Request, Result, Url, StatusCode};
        use reqwest::header::{HeaderMap, HeaderValue, HeaderName};
        use serde::de::DeserializeOwned;
        use std::cell::RefCell;
        use std::rc::Rc;
        use $crate::client::Executor;
    };
}
