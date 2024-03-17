extern crate proc_macro;
use proc_macro::TokenStream;

mod replace;
mod util;

/// Replace identifiers with arbitrary code.
///
/// # Syntax
///
/// ```rust,no_run
/// channels_macros::replace! {
///     replace: {                // <- This is the "replace section"
///         [                     // <- This is a "replace list"
///             (src1 => dst11)
///             (src2 => dst12)
///             // ...
///             (srcN => dst1N)
///         ]
///         [                     // <- This is a "replace list"
///             (src1 => dst21)
///             (src2 => dst22)
///             // ...
///             (srcN => dst2N)
///         ]
///         // ...
///         [                     // <- This is a "replace list"
///             (src1 => dstM1)
///             (src2 => dstM2)
///             // ...
///             (srcN => dstMN)
///         ]
///     }
///     code: {                   // <- This is the "code section"
///         // <actual code>
///     }
/// }
/// ```
///
/// The code inside the code section will be generated `M` times, as many times
/// as there are replace lists. Each time the code is generated, if any identifier
/// from its respective replace list is found, then it will be replaced with the
/// code following its `=>` arrow. For example if `src1` is found, the first time
/// it will be replaced with `dst11`, the second with `dst21`, and so on.
///
/// # Example
///
/// ```rust
/// channels_macros::replace! {
///     replace: {
///         [
///             (my_fn => my_fn_1)
///         ]
///         [
///             (my_fn => my_fn_2)
///         ]
///         [
///             (my_fn => my_fn_3)
///         ]
///     }
///     code: {
///         pub fn my_fn() -> &'static str {
///             stringify!(my_fn)
///         }
///     }
/// }
///
/// assert_eq!(my_fn_1(), "my_fn_1");
/// assert_eq!(my_fn_2(), "my_fn_2");
/// assert_eq!(my_fn_3(), "my_fn_3");
/// ```
///
/// The above will expand to 3 different version of the `my_fn` function all with
/// different names.
///
/// ```rust,no_run
/// pub fn my_fn_1() -> &'static str {
///     stringify!(my_fn_1)
/// }
///
/// pub fn my_fn_2() -> &'static str {
///     stringify!(my_fn_2)
/// }
///
/// pub fn my_fn_3() -> &'static str {
///     stringify!(my_fn_3)
/// }
/// ```
///
/// This means you can generate both synchronous code and asynchronous code and
/// await only where necessary.
///
/// # Example
///
/// ```rust,no_run
/// channels_macros::replace! {
///     replace: {
///         [
///             (_connect => connect_sync)
///             (async =>)
///             (await =>)
///             (TcpStream => std::net::TcpStream)
///         ]
///         [
///             (_connect => connect_async)
///             (async => async)
///             (await => .await)
///             (TcpStream => tokio::net::TcpStream)
///         ]
///     }
///     code: {
///         pub async fn _connect(addr: &str) -> TcpStream {
///             TcpStream::connect(addr) await .unwrap()
///         }
///     }
/// }
///
/// async fn async_fn() {
///     let stream = connect_async("127.0.0.1:8080").await;
///     // ...
/// }
///
/// fn sync_fn() {
///     let stream = connect_sync("127.0.0.1:8080");
///     // ...
/// }
/// ```
#[proc_macro]
pub fn replace(item: TokenStream) -> TokenStream {
	self::replace::entry(item)
}
