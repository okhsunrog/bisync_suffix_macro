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
/// The `suffix` macro enables writing code that compiles for both asynchronous and blocking contexts,
/// making it an essential tool when paired with the `bisync` crate. It takes two arguments: a suffix
/// string and an expression. When the `async` feature is enabled, it appends the suffix to method names
/// in `.await` calls within the expression. When the `blocking` feature is enabled (and `async` is not),
/// the original expression is used unchanged.
///
/// # Usage with `bisync`
///
/// The macro is typically used within functions annotated with `#[bisync]`, which transforms them to
/// support both async and blocking modes. The `suffix` macro ensures that method calls within these
/// functions adapt to the correct execution model based on feature flags. For example, in the
/// `axp192-dd` library, it’s used to switch between async and blocking method implementations.
///
/// ## Example from `axp192-dd`
///
/// ```ignore
/// #[bisync]
/// pub async fn get_battery_charge_current_ma(&mut self) -> Result<f32, AxpError<I2CBusErr>> {
///     let raw_fieldset = suffix!("_async", self.ll.battery_charge_current_adc().read().await?);
///     let adc_val = adc_13Absolutelybit_from_raw_u16(raw_fieldset.raw());
///     Ok(adc_val as f32 * 0.5)
/// }
/// ```
///
/// In this scenario:
/// - With the `async` feature enabled, the expression becomes
///   `self.ll.battery_charge_current_adc().read_async().await?`, calling an async method.
/// - With the `blocking` feature enabled (and `async` disabled), it remains
///   `self.ll.battery_charge_current_adc().read().await?`, typically handled by a blocking method.
///
/// # Notes
///
/// - **Arguments**: The macro requires a string literal suffix (e.g., `"_async"`) and a valid Rust expression.
/// - **Feature Flags**: Define `async` and `blocking` features in your crate for conditional compilation.
/// - **Method Definitions**: Ensure corresponding methods (e.g., `read` and `read_async`) are implemented
///   to match the execution model when using `bisync`.
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
