use core::fmt;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, LitInt, Path,
    PathArguments, PathSegment, Token, Type, TypePath,
};

#[derive(Debug)]
struct Input {
    literal: LitInt,
    _comma: Token![,],
    types: Punctuated<Type, Token![,]>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            literal: input.parse()?,
            _comma: input.parse()?,
            types: {
                let types = input.parse_terminated(Type::parse, Token![,])?;
                if types.empty_or_trailing() {
                    return Err(syn::Error::new(types.span(), "provide at least one type"));
                }
                types
            },
        })
    }
}

struct NumericType {
    signed: bool,
    bits: u8,
}

impl fmt::Display for NumericType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", if self.signed { "i" } else { "u" }, self.bits)
    }
}

impl NumericType {
    fn fits_u128(&self, n: u128) -> bool {
        let max = if self.signed {
            2u128.pow((self.bits - 1) as _)
        } else {
            2u128.pow(self.bits as _)
        };
        n <= max
    }

    fn fits_i128(&self, n: i128) -> bool {
        if !self.signed {
            return false;
        }
        let max = 2i128.checked_pow((self.bits - 1) as _).unwrap();
        let min = (-max).checked_sub(1).unwrap();

        min <= n && n <= max
    }
}

enum Value {
    Positive(u128),
    Negative(i128),
}

fn min_type_impl(input: Input) -> syn::Result<TokenStream> {
    let number = input
        .literal
        .base10_parse::<u128>()
        .map(Value::Positive)
        .or_else(|_| input.literal.base10_parse().map(Value::Negative))?;

    Ok(input
        .types
        .into_iter()
        .map(|ty| match &ty {
            Type::Path(TypePath {
                qself: None,
                path:
                    Path {
                        leading_colon: None,
                        segments,
                    },
            }) if segments.len() == 1 => match &segments[0] {
                PathSegment {
                    ident,
                    arguments: PathArguments::None,
                } => {
                    let ident = ident.to_string();
                    let (signedness, bits) = ident.split_at(1);
                    Ok(NumericType {
                        signed: match signedness {
                            "u" => false,
                            "i" => true,
                            _ => todo!(),
                        },
                        bits: bits.parse().unwrap(),
                    })
                }
                _ => Err(syn::Error::new(ty.span(), "invalid type: {ty}")),
            },
            _ => Err(syn::Error::new(ty.span(), "invalid type: {ty}")),
        })
        .filter(|ty| {
            let Ok(ty) = ty else { return true };
            match number {
                Value::Positive(n) => ty.fits_u128(n),
                Value::Negative(n) => ty.fits_i128(n),
            }
        })
        .try_fold(None::<NumericType>, |prev, current| {
            let Some(prev) = prev else {
                return current.map(Some);
            };
            let current = current?;
            if prev.bits > current.bits {
                Ok(Some(current))
            } else {
                Ok(Some(prev))
            }
        })?
        .ok_or_else(|| {
            syn::Error::new(
                Span::mixed_site(),
                "number does not fit any of the provided types",
            )
        })?
        .to_string()
        .parse()
        .unwrap())
}

#[proc_macro]
pub fn min_type(input: TokenStream) -> TokenStream {
    match min_type_impl(parse_macro_input!(input as Input)) {
        Ok(tt) => tt,
        Err(e) => e.into_compile_error().into(),
    }
}
