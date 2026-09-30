use std::ops::Deref;

/// Prefixes of the type sorted by name, so the generated code does not depend on hash map order.
pub(crate) fn sorted_prefixes<P, N>(
  prefix_mappings: impl IntoIterator<Item = (P, N)>,
) -> Vec<(String, String)>
where
  P: ToString,
  N: Deref<Target = str>,
{
  let mut prefixes = prefix_mappings
    .into_iter()
    .map(|(prefix, namespace)| (prefix.to_string(), namespace.to_string()))
    .collect::<Vec<_>>();
  prefixes.sort();
  prefixes
}

/// Compacts an IRI with the longest matching namespace, e.g. `http://example.com/name` into
/// `ex:name`. The IRI is kept as is when no namespace matches.
pub(crate) fn compact(iri: &str, prefixes: &[(String, String)]) -> String {
  prefixes
    .iter()
    .filter_map(|(prefix, namespace)| {
      iri
        .strip_prefix(namespace.as_str())
        .filter(|local_name| !local_name.is_empty())
        .map(|local_name| (namespace.len(), format!("{prefix}:{local_name}")))
    })
    .max_by_key(|(namespace_length, _)| *namespace_length)
    .map(|(_, compacted)| compacted)
    .unwrap_or_else(|| iri.to_string())
}
