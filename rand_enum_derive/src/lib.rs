
extern crate proc_macro;
use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/*use rand::{
	Rng,
	distributions::{Distribution, Standard},
};*/

#[proc_macro_derive(EnumRand)]
pub fn enum_distribution(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	
	let name = &input.ident;
    let gen = &input.generics;
	let (impl_generics, ty_generics, where_clause) = gen.split_for_impl();
	
	let variants = match input.data {
        syn::Data::Enum(ref v) => &v.variants,
        _ => panic!("EnumRand can only be used with enums"),
	};
	
	let mut idents = Vec::with_capacity(variants.len());
	for v in variants { idents.push(v.ident.clone()); }
	
	let last = idents.pop().expect("Enum must have at least 1 field");
	let num_enums = idents.len();
	
	let mut n = Vec::with_capacity(num_enums);
	for x in 0..num_enums{ n.push(x); }
	
	let result = quote! {
		impl #impl_generics rand::distributions::Distribution<#name> for rand::distributions::Standard #ty_generics #where_clause {
			fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> #name {
				match rng.gen_range(0, #num_enums) {
					#( #n => #name::#idents, )*
					_ => #name::#last
				}
			}
		}
	};
	result.into()
}