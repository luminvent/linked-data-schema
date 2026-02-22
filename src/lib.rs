mod linked_data_schema_field_visitor;

pub use linked_data_schema_derive::LinkedDataSchema;
pub use linked_data_schema_field_visitor::LinkedDataSchemaFieldVisitor;
use rudof_rdf::rdf_core::Rdf;
use shacl_ast::ShaclSchema;
use shacl_ast::component::Component;

pub mod reexports {
  pub use iri_s;
  pub use prefixmap;
  pub use rudof_rdf;
  pub use shacl_ast;
  pub use shacl_rdf;
  pub use uuid;
}

pub trait LinkedDataSchema {
  fn shacl<RDF: Rdf>() -> ShaclSchema<RDF>;

  fn components() -> Vec<Component>;
}

#[macro_export]
macro_rules! print_linked_data_schema_for {
  ( $x:ty ) => {
    let schema = <$x>::shacl();
    {
      use ::linked_data_schema::reexports::rudof_rdf::rdf_core::RDFFormat::Turtle;
      use ::linked_data_schema::reexports::rudof_rdf::rdf_impl::InMemoryGraph;
      use ::linked_data_schema::reexports::shacl_rdf::ShaclWriter;

      let mut shacl_writer = ShaclWriter::<InMemoryGraph>::default();
      shacl_writer.write(&schema).unwrap();

      let mut cursor = Cursor::new(Vec::new());

      shacl_writer.serialize(&Turtle, &mut cursor).unwrap();

      let content = cursor.into_inner();
      let s = String::from_utf8(content).unwrap();
      println!("{}", s);
    }
  };
}
