use proc_macro_crate::FoundCrate;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, DeriveInput, Generics, Ident, ItemStruct, LitInt, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct Crate;

impl ToTokens for Crate {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let found_crate =
            proc_macro_crate::crate_name("rustorio-engine").expect("Failed to get crate name");
        match found_crate {
            FoundCrate::Itself => quote! {crate}.to_tokens(tokens),
            FoundCrate::Name(name) => {
                let crate_ident = Ident::new(&name, Span::call_site());
                quote! {::#crate_ident}.to_tokens(tokens);
            }
        }
    }
}

struct RecipeItemAttrArgs(LitInt, Type);

impl Parse for RecipeItemAttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let amount = content.parse()?;
        let _ = content.parse::<Token![,]>()?;
        let ty = content.parse()?;
        Ok(Self(amount, ty))
    }
}

struct RecipeItemsAttr(Punctuated<RecipeItemAttrArgs, Token![,]>);

impl Parse for RecipeItemsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input.parse_terminated(RecipeItemAttrArgs::parse, Token![,])?,
        ))
    }
}

struct RecipeItemList {
    item_list: Vec<(u32, Type)>,
    item_type_ident: Ident,
    amount_const_ident: Ident,
}

impl RecipeItemList {
    fn new(
        attr: &Attribute,
        attr_name: &str,
        item_type_name: &str,
        amount_const_name: &str,
    ) -> Self {
        let Ok(inner) = attr.parse_args::<RecipeItemsAttr>() else {
            panic!("Invalid \"{attr_name}\" args");
        };

        let per_type = inner
            .0
            .iter()
            .map(|RecipeItemAttrArgs(lit, ty)| {
                let amount = lit
                    .base10_parse::<u32>()
                    .unwrap_or_else(|_| panic!("Invalid amount in \"{attr_name}\" args"));
                (amount, ty.to_owned())
            })
            .collect::<Vec<_>>();
        let item_type_ident = Ident::new(item_type_name, Span::call_site());
        let amount_const_ident = Ident::new(amount_const_name, Span::call_site());

        Self {
            item_list: per_type,
            item_type_ident,
            amount_const_ident,
        }
    }

    fn new_inputs(attr: &Attribute) -> Self {
        Self::new(attr, "recipe_inputs", "Inputs", "INPUT_AMOUNTS")
    }

    fn new_outputs(attr: &Attribute) -> Self {
        Self::new(attr, "recipe_outputs", "Outputs", "OUTPUT_AMOUNTS")
    }

    fn generate_recipe_direction(&self, amount_type_name: &str) -> TokenStream {
        let RecipeItemList {
            item_list,
            item_type_ident,
            amount_const_ident,
        } = self;

        let amount_type_ident = Ident::new(amount_type_name, Span::call_site());
        let amount_types = item_list.iter().map(|_| quote! {u32}).collect::<Vec<_>>();
        let amounts = item_list.iter().map(|(amount, _)| amount);

        let recipe_items = item_list
            .iter()
            .map(|(_, ty)| quote! {#Crate::resources::Resource<#ty>});

        quote! {
            type #item_type_ident = (#(#recipe_items,)*);

            type #amount_type_ident = (#(#amount_types,)*);
            const #amount_const_ident: (#(#amount_types,)*) = (#(#amounts,)*);
        }
    }

    fn generate_recipe_new_method(
        &self,
        new_fn_name: &str,
        implementing_trait: TokenStream,
    ) -> TokenStream {
        let RecipeItemList {
            item_list,
            item_type_ident,
            amount_const_ident: _,
        } = self;

        let new_fn_ident = Ident::new(new_fn_name, Span::call_site());
        let new_values = item_list
            .iter()
            .map(|_| quote! {#Crate::resources::Resource::new_empty()});

        quote! {
            fn #new_fn_ident() -> <Self as #implementing_trait>::#item_type_ident {
                (#(#new_values,)*)
            }
        }
    }

    fn generate_recipe_new_bundle_method(&self, new_fn_name: &str) -> TokenStream {
        let RecipeItemList {
            item_list,
            item_type_ident: _,
            amount_const_ident: _,
        } = self;

        let new_fn_ident = Ident::new(new_fn_name, Span::call_site());
        let new_values = item_list
            .iter()
            .map(|(amount, ty)| quote! {#Crate::resources::bundle::<#ty, #amount>()});

        quote! {
            fn #new_fn_ident() -> <Self as RecipeEx>::OutputBundle {
                (#(#new_values,)*)
            }
        }
    }

    fn generate_recipe_iter_method(
        &self,
        iter_fn_name: &str,
        implementing_trait: TokenStream,
    ) -> TokenStream {
        let RecipeItemList {
            item_list,
            item_type_ident,
            amount_const_ident,
        } = self;

        let iter_fn_ident = Ident::new(iter_fn_name, Span::call_site());
        let iter_values = item_list
            .iter()
            .enumerate()
            .map(|(i, (_amount, resource_type))| {
                let i = LitInt::new(&i.to_string(), Span::call_site());
                quote! {(
                    <#resource_type as #Crate::ResourceType>::NAME,
                    <Self as #implementing_trait>::#amount_const_ident.#i,
                    #Crate::resources::resource_amount_mut(&mut items.#i)
                )}
            });

        quote! {
            fn #iter_fn_ident(
                items: &mut <Self as #implementing_trait>::#item_type_ident
            ) -> impl Iterator<Item = (&'static str, u32, &mut u32)> {
                [#(#iter_values,)*].into_iter()
            }
        }
    }

    fn generate_bundle_type(&self) -> TokenStream {
        let RecipeItemList {
            item_list,
            item_type_ident: _,
            amount_const_ident: _,
        } = self;

        let bundle_items = item_list
            .iter()
            .map(|(amount, ty)| quote! {#Crate::resources::Bundle<#ty, #amount>});

        quote! {
            (#(#bundle_items,)*)
        }
    }
}

struct RecipeDetails {
    name: Ident,
    generics: Generics,

    inputs: RecipeItemList,
    outputs: RecipeItemList,
    ticks: LitInt,
}

impl RecipeDetails {
    fn from_input(input: DeriveInput) -> Self {
        Self::from_attrs(&input.attrs, input.ident, input.generics)
    }

    fn from_attrs(attrs: &[Attribute], name: Ident, generics: Generics) -> Self {
        let mut inputs = None;
        let mut outputs = None;
        let mut ticks = None;
        for attr in attrs {
            if attr.path().is_ident("recipe_inputs") {
                inputs = Some(RecipeItemList::new_inputs(attr));
            } else if attr.path().is_ident("recipe_outputs") {
                outputs = Some(RecipeItemList::new_outputs(attr));
            } else if attr.path().is_ident("recipe_ticks") {
                ticks = Some(
                    attr.parse_args::<LitInt>()
                        .expect("Invalid \"recipe_ticks\" value"),
                );
            }
        }
        let inputs = inputs.expect("Missing \"recipe_inputs\" attribute");
        let outputs = outputs.expect("Missing \"recipe_outputs\" attribute");
        let ticks = ticks.expect("Missing \"recipe_ticks\" attribute");

        Self {
            name,
            generics,
            inputs,
            outputs,
            ticks,
        }
    }

    fn generate_doc(&self) -> String {
        let mut doc_lines = Vec::new();

        doc_lines.push("### Input".to_string());
        for (amount, ty) in &self.inputs.item_list {
            let type_str = quote! { #ty }.to_string();
            doc_lines.push(format!("- [`{type_str}`] :  {amount}\n"));
        }
        doc_lines.push("### Output".to_string());
        for (amount, ty) in &self.outputs.item_list {
            let type_str = quote! { #ty }.to_string();
            doc_lines.push(format!("- [`{type_str}`] :  {amount}\n"));
        }
        doc_lines.push("### Time".to_string());

        doc_lines.push(format!("- **Ticks**: {}\n", self.ticks));

        doc_lines.join("\n")
    }

    fn recipe_impl(&self) -> TokenStream {
        let implementing_trait_path = quote! {#Crate::recipe::Recipe};
        let inputs_stream = self.inputs.generate_recipe_direction("InputAmountsType");
        let outputs_stream = self.outputs.generate_recipe_direction("OutputAmountsType");

        let new_inputs_method_stream = self
            .inputs
            .generate_recipe_new_method("new_inputs", implementing_trait_path.clone());
        let new_outputs_method_stream = self
            .outputs
            .generate_recipe_new_method("new_outputs", implementing_trait_path.clone());

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let name = &self.name;
        let ticks = &self.ticks;
        quote! {
            impl #impl_generics #Crate::recipe::Recipe for #name #ty_generics #where_clause {
                const TIME: u64 = #ticks;

                #new_inputs_method_stream
                #new_outputs_method_stream

                #inputs_stream
                #outputs_stream
            }
        }
    }

    fn recipe_ex_impl(&self) -> TokenStream {
        let implementing_trait_path = quote! {#Crate::recipe::Recipe};
        let input_bundle_type = self.inputs.generate_bundle_type();
        let output_bundle_type = self.outputs.generate_bundle_type();
        let new_output_bundle_method_stream = self
            .outputs
            .generate_recipe_new_bundle_method("new_output_bundle");
        let iter_inputs_method_stream = self
            .inputs
            .generate_recipe_iter_method("iter_inputs", implementing_trait_path.clone());
        let iter_outputs_method_stream = self
            .outputs
            .generate_recipe_iter_method("iter_outputs", implementing_trait_path.clone());
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let name = &self.name;
        quote! {
            impl #impl_generics #Crate::recipe::RecipeEx for #name #ty_generics #where_clause {
                type InputBundle = #input_bundle_type;
                type OutputBundle = #output_bundle_type;
                #new_output_bundle_method_stream
                #iter_inputs_method_stream
                #iter_outputs_method_stream
            }
        }
    }
}

#[proc_macro_derive(Recipe, attributes(recipe_inputs, recipe_outputs, recipe_ticks))]
pub fn derive_recipe(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let recipe_info = RecipeDetails::from_input(input);
    let output = recipe_info.recipe_impl();
    proc_macro::TokenStream::from(output)
}

#[proc_macro_derive(RecipeEx, attributes(recipe_inputs, recipe_outputs, recipe_ticks))]
pub fn derive_recipe_ex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let recipe_info = RecipeDetails::from_input(input);
    let output = recipe_info.recipe_ex_impl();
    proc_macro::TokenStream::from(output)
}

/// Generates documentation for a recipe based on its inputs and outputs.
/// The generated documentation is appended to any existing documentation on the struct.
#[proc_macro_attribute]
pub fn recipe_doc(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = parse_macro_input!(input as ItemStruct);
    let recipe_info =
        RecipeDetails::from_attrs(&item.attrs, item.ident.clone(), item.generics.clone());

    let generated_doc = recipe_info.generate_doc();
    let doc_attr: Attribute = syn::parse_quote! {
        #[doc = #generated_doc]
    };

    // Insert the generated doc at the beginning of the attributes
    item.attrs.push(doc_attr);

    quote! { #item }.into()
}

struct TechnologyDetails {
    name: Ident,
    generics: Generics,
    research_inputs: RecipeItemList,
    point_recipe_time: LitInt,
    research_point_cost: LitInt,
}

impl TechnologyDetails {
    fn from_derive(input: DeriveInput) -> Self {
        Self::from_attrs(&input.attrs, input.ident, input.generics)
    }

    fn from_attrs(attrs: &[Attribute], name: Ident, generics: Generics) -> Self {
        let mut research_inputs = None;
        let mut research_point_cost = None;
        let mut research_ticks = None;
        for attr in attrs {
            if attr.path().is_ident("research_inputs") {
                research_inputs = Some(RecipeItemList::new_inputs(attr));
            } else if attr.path().is_ident("research_point_cost") {
                research_point_cost = Some(
                    attr.parse_args::<LitInt>()
                        .expect("Invalid \"research_point_cost\" value"),
                );
            } else if attr.path().is_ident("research_ticks") {
                research_ticks = Some(
                    attr.parse_args::<LitInt>()
                        .expect("Invalid \"research_ticks\" value"),
                );
            }
        }
        let research_inputs = research_inputs.expect("Missing \"research_inputs\" attribute");
        let research_point_cost =
            research_point_cost.expect("Missing \"research_point_cost\" attribute");
        let research_ticks = research_ticks.expect("Missing \"research_ticks\" attribute");

        Self {
            name,
            generics,
            research_inputs,
            research_point_cost,
            point_recipe_time: research_ticks,
        }
    }

    fn generate_doc(&self) -> String {
        let mut doc_lines = Vec::new();

        doc_lines.push("### Cost".to_string());
        for (amount, ty) in &self.research_inputs.item_list {
            let type_str = quote! { #ty }.to_string();
            doc_lines.push(format!("- [`{type_str}`] :  {amount}\n"));
        }

        doc_lines.push(format!("**Ticks**: {}\n", self.point_recipe_time));

        doc_lines.push(format!(
            "**Research points required**: {}",
            self.research_point_cost
        ));

        doc_lines.join("\n")
    }

    fn technology_impl(&self) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let name = &self.name;

        let inputs_stream = self
            .research_inputs
            .generate_recipe_direction("InputAmountsType");
        let research_point_cost = &self.research_point_cost;
        let point_recipe_time = &self.point_recipe_time;

        let input_bundle_type = self.research_inputs.generate_bundle_type();

        let implementing_trait_path = quote! {#Crate::research::TechnologyEx};

        let new_inputs_method_stream = self
            .research_inputs
            .generate_recipe_new_method("new_inputs", implementing_trait_path.clone());

        let iter_inputs_method_stream = self
            .research_inputs
            .generate_recipe_iter_method("iter_inputs", implementing_trait_path.clone());

        quote! {
            impl #impl_generics #Crate::research::TechnologyEx for #name #ty_generics #where_clause {
                #inputs_stream
                const POINT_RECIPE_TIME: u64 = #point_recipe_time;
                const REQUIRED_RESEARCH_POINTS_EX: u32 = #research_point_cost;
                type InputBundle = #input_bundle_type;

                #new_inputs_method_stream
                #iter_inputs_method_stream
            }
        }
    }
}

#[proc_macro_derive(
    TechnologyEx,
    attributes(research_inputs, research_point_cost, research_ticks)
)]
pub fn derive_technology(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let tech_info = TechnologyDetails::from_derive(input);
    let output = tech_info.technology_impl();
    proc_macro::TokenStream::from(output)
}

/// Generates documentation for a technology struct based on its `research_inputs` and `research_ticks` attributes.
/// The generated documentation is appended to any existing documentation on the struct.
#[proc_macro_attribute]
pub fn technology_doc(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = parse_macro_input!(input as ItemStruct);

    // Parse the technology details from the struct's attributes
    let tech_info =
        TechnologyDetails::from_attrs(&item.attrs, item.ident.clone(), item.generics.clone());

    // Generate the documentation
    let generated_doc = tech_info.generate_doc();
    let doc_attr: Attribute = syn::parse_quote! {
        #[doc = #generated_doc]
    };

    // Insert the generated doc at the beginning of the attributes
    item.attrs.push(doc_attr);

    quote! { #item }.into()
}
