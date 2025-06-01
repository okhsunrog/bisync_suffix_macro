//! `bisync_suffix_macro` provides a procedural macro to conditionally append suffixes to method names
//! in `.await` expressions, enabling dual support for asynchronous and blocking code paths.
//!
//! This macro is designed to work seamlessly with the `bisync` crate, which allows functions to support
//! both async and blocking execution models based on feature flags. It is particularly useful in libraries
//! like `axp192-dd`, where a single codebase needs to provide both async and blocking APIs.
//!
//! For detailed usage, see the [`suffix`] macro documentation.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, ExprAwait, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    visit_mut::{self, VisitMut},
};

/// A procedural macro to conditionally append a suffix to method names in `.await` expressions.
///
/// The `suffix` macro is designed to work in conjunction with the `#[bisync]` attribute from the `bisync` crate,
/// enabling functions to support both asynchronous and blocking execution models based on feature flags.
///
/// When used within a function annotated with `#[bisync]`, the `suffix` macro ensures that method calls
/// in `.await` expressions are adapted to the correct execution model:
///
/// - With the `async` feature enabled, it appends the specified suffix (e.g., `"_async"`) to method names in `.await` calls,
///   ensuring the async version of the method is invoked.
/// - With the `blocking` feature enabled (and `async` disabled), the original method name is preserved without the suffix,
///   and the `#[bisync]` attribute removes the `.await` from the expression, transforming the function into a synchronous
///   version that calls the blocking method directly.
///
/// # Usage with `bisync`
///
/// Below is an example from the `axp192-dd` library:
///
/// ```ignore
/// #[bisync]
/// pub async fn get_battery_charge_current_ma(&mut self) -> Result<f32, AxpError<I2CBusErr>> {
///     let raw_fieldset = suffix!("_async", self.ll.battery_charge_current_adc().read().await?);
///     let adc_val = adc_13bit_from_raw_u16(raw_fieldset.raw());
///     Ok(adc_val as f32 * 0.5)
/// }
/// ```
///
/// In this scenario:
/// - **Async Context**: When the `async` feature is enabled, the `suffix` macro transforms the expression to
///   `self.ll.battery_charge_current_adc().read_async().await?`, calling the asynchronous `read_async` method.
/// - **Blocking Context**: When the `blocking` feature is enabled (and `async` disabled), the `suffix` macro leaves the
///   method name as `read` (no suffix is appended), and the `#[bisync]` attribute removes the `.await`, resulting in
///   `self.ll.battery_charge_current_adc().read()?`. Here, `read` is expected to be a blocking method provided by the
///   underlying implementation.
///
/// # Notes
///
/// - **Arguments**: The macro requires a string literal suffix (e.g., `"_async"`) and a valid Rust expression.
/// - **Feature Flags**: Ensure that `async` and `blocking` features are defined in your crate for conditional compilation.
/// - **Method Definitions**: Corresponding methods (e.g., `read` for blocking and `read_async` for async) must be
///   implemented to match the execution model when using `bisync`.
#[proc_macro]
pub fn suffix(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as SuffixMacroInput);
    let suffix_value = parsed_input.suffix_str.value();

    let mut async_expr_transformed = parsed_input.expr.clone();
    let blocking_expr_original = parsed_input.expr;

    let mut suffixer = AwaitMethodSuffixer {
        suffix: &suffix_value,
    };
    suffixer.visit_expr_mut(&mut async_expr_transformed);

    let output = quote! {
        {
            #[cfg(feature = "async")]
            {
                #async_expr_transformed
            }
            #[cfg(all(feature = "blocking", not(feature = "async")))]
            {
                #blocking_expr_original
            }
        }
    };

    output.into()
}

struct SuffixMacroInput {
    suffix_str: LitStr,
    _comma: Token![,],
    expr: Expr,
}

impl Parse for SuffixMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(SuffixMacroInput {
            suffix_str: input.parse()?,
            _comma: input.parse()?,
            expr: input.parse()?,
        })
    }
}

struct AwaitMethodSuffixer<'a> {
    suffix: &'a str,
}

impl<'a> VisitMut for AwaitMethodSuffixer<'a> {
    fn visit_expr_await_mut(&mut self, expr_await: &mut ExprAwait) {
        if let Expr::MethodCall(method_call) = &mut *expr_await.base {
            let original_method_ident = method_call.method.clone();
            let new_method_name_str = format!("{}{}", original_method_ident, self.suffix);
            method_call.method = Ident::new(&new_method_name_str, original_method_ident.span());
        }
        visit_mut::visit_expr_await_mut(self, expr_await);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_mut::visit_expr_mut(self, expr);
    }
}
