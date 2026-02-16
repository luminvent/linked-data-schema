use linked_data_core::{RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant, TokenGenerator};
use proc_macro_error::proc_macro_error;
use proc_macro2::{Literal, TokenStream};
use quote::ToTokens;
use syn::DeriveInput;
use uuid::Uuid;

#[proc_macro_error]
#[proc_macro_derive(LinkedDataSchema, attributes(ld))]
pub fn derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let raw_input = syn::parse_macro_input!(item as DeriveInput);
  let linked_data_type: RdfType<Schema> = RdfType::from_derive(raw_input);

  let mut output = TokenStream::new();
  linked_data_type.to_tokens(&mut output);
  output.into()
}

#[derive(Debug)]
struct Schema;

impl TokenGenerator for Schema {
  fn generate_type_tokens(linked_data_type: &RdfType<Self>, tokens: &mut TokenStream) {
    let implementations = match linked_data_type {
      RdfType::Enum(rdf_enum) => quote::quote! {#rdf_enum},
      RdfType::Struct(rdf_struct) => quote::quote! {#rdf_struct},
    };

    tokens.extend(implementations)
  }

  fn generate_struct_tokens(rdf_struct: &RdfStruct<Self>, tokens: &mut TokenStream) {
    let type_iri = rdf_struct.type_iri().unwrap();

    let type_iri_shape = Literal::string(&format!("{}Shape", type_iri.as_str()));
    let type_iri = Literal::string(type_iri.as_str());

    let prefix_mapping = rdf_struct.prefix_mappings().clone();

    let insert_all_prefix_mapping = prefix_mapping
      .into_iter()
      .map(|(prefix, iri)| {
        let prefix = Literal::string(&prefix.to_string());
        let iri = Literal::string(&iri);

        quote::quote! {
          prefix_map.add_prefix(#prefix, iri!(#iri)).unwrap();
        }
      })
      .collect::<TokenStream>();

    let ident = &rdf_struct.ident;
    let fields = &rdf_struct.fields;

    let property_shapes_iris = fields
      .iter()
      .map(|field| {
        if let Some(predicate) = field.predicate() {
          let identifier = Literal::string(&format!("{}Field", predicate.as_str()));

          quote::quote! {
            RDFNode::iri(IriS::from_str(#identifier).unwrap()),
          }
        } else {
          quote::quote! {}
        }
      })
      .collect::<TokenStream>();

    let struct_blank_node = Literal::string(Uuid::new_v4().to_string().as_str());

    tokens.extend(quote::quote! {
      impl ::linked_data_schema::LinkedDataSchemaFieldVisitor for #ident {
        fn field_components() -> Vec<::linked_data_schema::reexports::shacl_ast::ast::component::Component> {
          Self::components()
        }

        fn type_iri_ref() -> Option<::linked_data_schema::reexports::prefixmap::IriRef> {
          use ::linked_data_schema::reexports::prefixmap::IriRef;
          use ::linked_data_schema::reexports::iri_s::iri;

          Some(IriRef::iri(iri!(#type_iri_shape)))
        }
      }

      impl ::linked_data_schema::LinkedDataSchema for #ident {
        fn shacl() -> ::linked_data_schema::reexports::shacl_ast::Schema<::linked_data_schema::reexports::srdf::SRDFGraph> {
          use ::linked_data_schema::{
            reexports::{
              iri_s::{IriS, iri},
              prefixmap::{PrefixMap, IriRef},
              shacl_ast::{
                ast::{
                  component::Component,
                  shape::Shape,
                  node_shape::NodeShape,
                  property_shape::PropertyShape,
                  target::Target,
                },
                Schema,
              },
              srdf::{
                RDFNode,
                SHACLPath,
              },
            },
            LinkedDataSchemaFieldVisitor,
          };
          use std::str::FromStr;
          use std::collections::HashMap;

          let mut prefix_map = PrefixMap::new();
          #insert_all_prefix_mapping

          let mut shapes = HashMap::default();

          let rdf_node_type_iri = RDFNode::iri(IriS::from_str(#type_iri_shape).unwrap());

          let property_shapes = vec![
            #property_shapes_iris
          ];

          let node_shape = NodeShape::new(rdf_node_type_iri.clone())
            .with_targets(vec![Target::TargetClass(RDFNode::iri(IriS::from_str(#type_iri).unwrap()))])
            .with_property_shapes(property_shapes);

          let _ = shapes.insert(RDFNode::BlankNode(#struct_blank_node.to_string()), Shape::NodeShape(Box::new(node_shape)));

          #(#fields)*

          Schema::default()
            .with_prefixmap(prefix_map)
            .with_shapes(shapes)
        }

        fn components() -> Vec<::linked_data_schema::reexports::shacl_ast::ast::component::Component> {
          use ::linked_data_schema::{
            reexports::{
              iri_s::iri,
              prefixmap::IriRef,
              shacl_ast::ast::component::Component,
            }
          };

          vec![
            Component::Datatype(IriRef::iri(iri!(#type_iri_shape))),
          ]
        }
      }
    })
  }

  fn generate_enum_tokens(r#enum: &RdfEnum<Self>, tokens: &mut TokenStream) {
    let _variants = &r#enum.variants;
    let ident = &r#enum.ident;

    tokens.extend(quote::quote! {
      impl ::linked_data_schema::LinkedDataSchema for #ident {
        fn shacl() -> ::linked_data_schema::reexports::shacl_ast::Schema<::linked_data_schema::reexports::srdf::SRDFGraph> {
          use ::linked_data_schema::reexports::{
            prefixmap::PrefixMap,
            shacl_ast::{
              ast::shape::Shape,
              Schema,
            },
            srdf::RDFNode,
          };
          use std::collections::HashMap;

          let prefix_map = PrefixMap::new();
          let shapes = HashMap::default();

          Schema::default()
            .with_prefixmap(prefix_map)
            .with_shapes(shapes)
        }
      }
    })
  }

  fn generate_variant_tokens(_variant: &RdfVariant<Self>, _tokens: &mut TokenStream) {
    todo!()
  }

  fn generate_field_tokens(field: &RdfField<Self>, tokens: &mut TokenStream) {
    if field.is_ignored() {
      return;
    }

    //if field.is_flattened() {

    //}

    if let Some(predicate) = field.predicate() {
      let identifier = Literal::string(&format!("{}Field", predicate.as_str()));
      let predicate = Literal::string(predicate.as_str());

      let field_type = &field.ty;

      tokens.extend(quote::quote! {
        let node = RDFNode::BlankNode(::linked_data_schema::reexports::uuid::Uuid::new_v4().to_string());

        let rdf_node_type_iri = RDFNode::iri(IriS::from_str(#identifier).unwrap());

        let property_shape = PropertyShape::new(
          rdf_node_type_iri,
          SHACLPath::iri(IriS::from_str(#predicate).unwrap()),
        ).with_components(<#field_type>::field_components());

        let _ = shapes.insert(node, Shape::PropertyShape(Box::new(property_shape)));
      })
    }
  }
}
