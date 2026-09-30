use linked_data_core::{
  PredicatePath, RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant, TokenGenerator,
};
use proc_macro_error3::{abort, proc_macro_error};
use proc_macro2::{Literal, TokenStream};
use quote::ToTokens;
use syn::DeriveInput;

mod prefixes;
mod to_schema;

use prefixes::{compact, sorted_prefixes};

#[proc_macro_error]
#[proc_macro_derive(LinkedDataSchema, attributes(ld))]
pub fn derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let raw_input = syn::parse_macro_input!(item as DeriveInput);
  let linked_data_type: RdfType<Schema> = RdfType::from_derive(raw_input);

  let mut output = TokenStream::new();
  linked_data_type.to_tokens(&mut output);
  output.into()
}

/// Implements `utoipa::ToSchema`, describing the type as a compacted JSON-LD document.
///
/// Requires the `utoipa` feature of `linked-data-schema`.
#[proc_macro_error]
#[proc_macro_derive(LinkedDataToSchema, attributes(ld))]
pub fn derive_to_schema(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let raw_input = syn::parse_macro_input!(item as DeriveInput);
  let linked_data_type: RdfType<to_schema::ToSchema> = RdfType::from_derive(raw_input);

  let mut output = TokenStream::new();
  linked_data_type.to_tokens(&mut output);
  output.into()
}

#[derive(Debug)]
struct Schema;

fn insert_all_prefix_mapping(prefixes: &[(String, String)]) -> TokenStream {
  prefixes
    .iter()
    .map(|(prefix, iri)| {
      let prefix = Literal::string(prefix);
      let iri = Literal::string(iri);

      quote::quote! {
        prefix_map.add_prefix(#prefix, iri!(#iri)).unwrap();
      }
    })
    .collect()
}

/// A SHACL path, as a `SHACLPath` expression, for the predicate path of a variant.
fn shacl_path(predicate_path: &PredicatePath) -> TokenStream {
  match predicate_path {
    PredicatePath::Predicate(predicate) => {
      let predicate = Literal::string(predicate.as_str());

      quote::quote! { SHACLPath::iri(IriS::from_str(#predicate).unwrap()) }
    }
    PredicatePath::ChainedPath {
      to_blank,
      from_blank,
    } => {
      let to_blank = Literal::string(to_blank.as_str());
      let from_blank = Literal::string(from_blank.as_str());

      quote::quote! {
        SHACLPath::sequence(vec![
          SHACLPath::iri(IriS::from_str(#to_blank).unwrap()),
          SHACLPath::iri(IriS::from_str(#from_blank).unwrap()),
        ])
      }
    }
  }
}

impl TokenGenerator for Schema {
  fn generate_type_tokens(linked_data_type: &RdfType<Self>, tokens: &mut TokenStream) {
    let implementations = match linked_data_type {
      RdfType::Enum(rdf_enum) => quote::quote! {#rdf_enum},
      RdfType::Struct(rdf_struct) => quote::quote! {#rdf_struct},
    };

    tokens.extend(implementations)
  }

  fn generate_struct_tokens(rdf_struct: &RdfStruct<Self>, tokens: &mut TokenStream) {
    let type_iri = rdf_struct.type_iri().expect("missing type iri");
    let prefixes = sorted_prefixes(rdf_struct.prefix_mappings().clone());

    let type_iri_shape = format!("{}Shape", type_iri.as_str());
    let type_iri = Literal::string(type_iri.as_str());

    let insert_all_prefix_mapping = insert_all_prefix_mapping(&prefixes);

    let ident = &rdf_struct.ident;

    // Property shapes are identified within their node shape, so structs sharing a predicate do
    // not overwrite each other's constraints once nested shapes are added to the same schema.
    let properties = rdf_struct
      .fields
      .iter()
      .filter(|field| !field.is_ignored())
      .filter_map(|field| {
        let predicate = field.predicate()?;
        let property_shape_iri = format!(
          "{type_iri_shape}/{}",
          compact(predicate.as_str(), &prefixes)
        );

        Some((
          Literal::string(&property_shape_iri),
          Literal::string(predicate.as_str()),
          &field.ty,
        ))
      })
      .collect::<Vec<_>>();

    let property_shapes_iris = properties.iter().map(|(property_shape_iri, _, _)| {
      quote::quote! {
        Object::Iri(IriS::from_str(#property_shape_iri).unwrap())
      }
    });

    let add_property_shapes = properties
      .iter()
      .map(|(property_shape_iri, predicate, field_type)| {
        quote::quote! {
          let property_shape_iri = Object::Iri(IriS::from_str(#property_shape_iri).unwrap());

          let property_shape = ASTPropertyShape::new(
            property_shape_iri.clone(),
            SHACLPath::iri(IriS::from_str(#predicate).unwrap()),
          ).with_components(<#field_type as LinkedDataSchemaFieldVisitor>::field_components());

          let _ = shapes.insert(property_shape_iri, ASTShape::PropertyShape(Box::new(property_shape)));

          <#field_type as LinkedDataSchemaFieldVisitor>::add_shapes(shapes);
        }
      })
      .collect::<TokenStream>();

    let type_iri_shape = Literal::string(&type_iri_shape);

    tokens.extend(quote::quote! {
      impl ::linked_data_schema::LinkedDataSchemaFieldVisitor for #ident {
        fn value_components() -> Vec<::linked_data_schema::reexports::shacl::ast::ASTComponent> {
          <Self as ::linked_data_schema::LinkedDataSchema>::components()
        }

        fn add_shapes(
          shapes: &mut ::std::collections::HashMap<
            ::linked_data_schema::reexports::rudof_rdf::rdf_core::term::Object,
            ::linked_data_schema::reexports::shacl::ast::ASTShape,
          >,
        ) {
          use ::linked_data_schema::{
            reexports::{
              iri_s::IriS,
              shacl::{
                ast::{ASTNodeShape, ASTPropertyShape, ASTShape},
                types::Target,
              },
              rudof_rdf::rdf_core::{SHACLPath, term::Object},
            },
            LinkedDataSchemaFieldVisitor,
          };
          use std::str::FromStr;

          let node_shape_iri = Object::Iri(IriS::from_str(#type_iri_shape).unwrap());

          // Types can reference each other: each node shape is added once.
          if shapes.contains_key(&node_shape_iri) {
            return;
          }

          let node_shape = ASTNodeShape::new(node_shape_iri.clone())
            .with_targets(vec![Target::Class(Object::Iri(IriS::from_str(#type_iri).unwrap()))])
            .with_property_shapes(vec![#(#property_shapes_iris),*]);

          let _ = shapes.insert(node_shape_iri, ASTShape::NodeShape(Box::new(node_shape)));

          #add_property_shapes
        }
      }

      impl ::linked_data_schema::LinkedDataSchema for #ident {
        fn shacl() -> ::linked_data_schema::reexports::shacl::ast::ASTSchema {
          use ::linked_data_schema::{
            reexports::{
              iri_s::iri,
              prefixmap::PrefixMap,
              shacl::ast::ASTSchema,
            },
            LinkedDataSchemaFieldVisitor,
          };
          use std::collections::HashMap;

          let mut prefix_map = PrefixMap::new();
          #insert_all_prefix_mapping

          let mut shapes = HashMap::default();
          <Self as LinkedDataSchemaFieldVisitor>::add_shapes(&mut shapes);

          ASTSchema::new()
            .with_prefixmap(prefix_map)
            .with_shapes(shapes)
        }

        fn components() -> Vec<::linked_data_schema::reexports::shacl::ast::ASTComponent> {
          use ::linked_data_schema::reexports::{
            iri_s::IriS,
            rudof_rdf::rdf_core::term::Object,
            shacl::ast::ASTComponent,
          };
          use std::str::FromStr;

          vec![
            ASTComponent::Node(Object::Iri(IriS::from_str(#type_iri_shape).unwrap())),
          ]
        }
      }
    })
  }

  fn generate_enum_tokens(rdf_enum: &RdfEnum<Self>, tokens: &mut TokenStream) {
    let ident = &rdf_enum.ident;
    let prefixes = sorted_prefixes(rdf_enum.prefix_mappings().clone());
    let insert_all_prefix_mapping = insert_all_prefix_mapping(&prefixes);

    // An enum has no type IRI, so its shapes are blank nodes named after it.
    let enum_shape = format!("{ident}Shape");

    let (components, add_shapes) = if rdf_enum.is_closed_list() {
      // Each unit variant is a resource identified by its own IRI.
      let resources = rdf_enum
        .variants
        .iter()
        .map(|variant| match variant.predicate_path() {
          PredicatePath::Predicate(iri) => Literal::string(iri.as_str()),
          PredicatePath::ChainedPath { .. } => abort!(
            ident.span(),
            "unit variants must be identified by a single IRI"
          ),
        });

      let components = quote::quote! {
        vec![
          ASTComponent::NodeKind(NodeKind::Iri),
          ASTComponent::In(vec![
            #(Value::Iri(IriRef::iri(IriS::from_str(#resources).unwrap())),)*
          ]),
        ]
      };

      (components, TokenStream::new())
    } else {
      // Each variant holds its value under its predicate path, and a value matches exactly one
      // variant.
      let variant_shapes = rdf_enum
        .variants
        .iter()
        .map(|variant| Literal::string(&format!("{enum_shape}_{}", variant.index)))
        .collect::<Vec<_>>();

      let add_variant_shapes = rdf_enum
        .variants
        .iter()
        .zip(&variant_shapes)
        .map(|(variant, variant_shape)| {
          let value_type = variant
            .ty
            .as_ref()
            .expect("unit variants are handled as closed list");
          let value_shape = Literal::string(&format!("{enum_shape}_{}_value", variant.index));
          let path = shacl_path(variant.predicate_path());

          quote::quote! {
            let value_shape = Object::BlankNode(#value_shape.to_string());

            let property_shape = ASTPropertyShape::new(value_shape.clone(), #path)
              .with_components(<#value_type as LinkedDataSchemaFieldVisitor>::field_components());
            let _ = shapes.insert(value_shape.clone(), ASTShape::PropertyShape(Box::new(property_shape)));

            let variant_shape = Object::BlankNode(#variant_shape.to_string());
            let node_shape = ASTNodeShape::new(variant_shape.clone())
              .with_property_shapes(vec![value_shape]);
            let _ = shapes.insert(variant_shape, ASTShape::NodeShape(Box::new(node_shape)));

            <#value_type as LinkedDataSchemaFieldVisitor>::add_shapes(shapes);
          }
        })
        .collect::<TokenStream>();

      let enum_shape = Literal::string(&enum_shape);

      let components = quote::quote! {
        vec![ASTComponent::Node(Object::BlankNode(#enum_shape.to_string()))]
      };

      let add_shapes = quote::quote! {
        let enum_shape = Object::BlankNode(#enum_shape.to_string());

        // Types can reference each other: each node shape is added once.
        if shapes.contains_key(&enum_shape) {
          return;
        }

        let node_shape = ASTNodeShape::new(enum_shape.clone()).with_components(vec![
          ASTComponent::Xone(vec![
            #(Object::BlankNode(#variant_shapes.to_string()),)*
          ]),
        ]);
        let _ = shapes.insert(enum_shape, ASTShape::NodeShape(Box::new(node_shape)));

        #add_variant_shapes
      };

      (components, add_shapes)
    };

    tokens.extend(quote::quote! {
      impl ::linked_data_schema::LinkedDataSchemaFieldVisitor for #ident {
        fn value_components() -> Vec<::linked_data_schema::reexports::shacl::ast::ASTComponent> {
          <Self as ::linked_data_schema::LinkedDataSchema>::components()
        }

        #[allow(unused_variables)]
        fn add_shapes(
          shapes: &mut ::std::collections::HashMap<
            ::linked_data_schema::reexports::rudof_rdf::rdf_core::term::Object,
            ::linked_data_schema::reexports::shacl::ast::ASTShape,
          >,
        ) {
          #[allow(unused_imports)]
          use ::linked_data_schema::{
            reexports::{
              iri_s::IriS,
              shacl::ast::{ASTComponent, ASTNodeShape, ASTPropertyShape, ASTShape},
              rudof_rdf::rdf_core::{SHACLPath, term::Object},
            },
            LinkedDataSchemaFieldVisitor,
          };
          #[allow(unused_imports)]
          use std::str::FromStr;

          #add_shapes
        }
      }

      impl ::linked_data_schema::LinkedDataSchema for #ident {
        fn shacl() -> ::linked_data_schema::reexports::shacl::ast::ASTSchema {
          #[allow(unused_imports)]
          use ::linked_data_schema::{
            reexports::{
              iri_s::iri,
              prefixmap::PrefixMap,
              shacl::ast::ASTSchema,
            },
            LinkedDataSchemaFieldVisitor,
          };
          use std::collections::HashMap;

          #[allow(unused_mut)]
          let mut prefix_map = PrefixMap::new();
          #insert_all_prefix_mapping

          let mut shapes = HashMap::default();
          <Self as LinkedDataSchemaFieldVisitor>::add_shapes(&mut shapes);

          ASTSchema::new()
            .with_prefixmap(prefix_map)
            .with_shapes(shapes)
        }

        fn components() -> Vec<::linked_data_schema::reexports::shacl::ast::ASTComponent> {
          #[allow(unused_imports)]
          use ::linked_data_schema::reexports::{
            iri_s::IriS,
            prefixmap::IriRef,
            rudof_rdf::rdf_core::term::Object,
            shacl::{
              ast::ASTComponent,
              types::{NodeKind, Value},
            },
          };
          #[allow(unused_imports)]
          use std::str::FromStr;

          #components
        }
      }
    })
  }

  // Variants are generated by their enum, which identifies their shapes.
  fn generate_variant_tokens(_variant: &RdfVariant<Self>, _tokens: &mut TokenStream) {}

  // Fields are generated by their struct, which identifies their property shapes.
  fn generate_field_tokens(_field: &RdfField<Self>, _tokens: &mut TokenStream) {}
}
