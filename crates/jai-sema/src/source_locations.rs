//! Source directives and deferred caller defaults share checked retained origins.
use super::*;
use jai_source::{SourceRecord, SourceSpan};
use jai_types::LayoutEngine;

pub(crate) struct SourcePoint<'a> {
    pub(crate) filename: &'a str,
    pub(crate) directory: Option<&'a str>,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

pub(crate) fn source_point(
    record: &SourceRecord,
    location: SourceSpan,
) -> Result<SourcePoint<'_>, Diagnostic> {
    let span = location.span;
    if record.id() != location.source
        || span.start > span.end
        || span.end > record.text().len()
        || !record.text().is_char_boundary(span.start)
        || !record.text().is_char_boundary(span.end)
    {
        return Err(Diagnostic::at_source(
            location,
            "source span is outside its retained source record",
        ));
    }
    let filename = record.path().to_str().ok_or_else(|| {
        Diagnostic::at_source(
            location,
            "source filename cannot be represented as a UTF-8 string",
        )
    })?;
    let prefix = &record.text()[..span.start];
    Ok(SourcePoint {
        filename,
        directory: record
            .path()
            .parent()
            .filter(|directory| !directory.as_os_str().is_empty())
            .and_then(|directory| directory.to_str()),
        line: prefix.bytes().filter(|&byte| byte == b'\n').count() + 1,
        column: prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
    })
}

fn coordinate(value: usize, location: SourceSpan) -> Result<IntExpr, Diagnostic> {
    IntegerValue::checked(IntegerType::S64, value as i128)
        .map(IntExpr::constant)
        .ok_or_else(|| Diagnostic::at_source(location, "source coordinate exceeds s64"))
}

pub(crate) fn location_constant(
    record: &SourceRecord,
    location: SourceSpan,
    ty: TypeId,
    types: &TypeRegistry,
) -> Result<jai_ir::ConstantValue, Diagnostic> {
    let point = source_point(record, location)?;
    let integer = |value: usize| {
        IntegerValue::checked(IntegerType::S64, value as i128)
            .map(|value| jai_ir::ConstantValue {
                ty: types.scalar(ScalarType::Int(IntegerType::S64)),
                kind: jai_ir::ConstantKind::Int(value),
            })
            .ok_or_else(|| Diagnostic::at_source(location, "source coordinate exceeds s64"))
    };
    Ok(jai_ir::ConstantValue {
        ty,
        kind: jai_ir::ConstantKind::Record(vec![
            jai_ir::ConstantValue {
                ty: types.string(),
                kind: jai_ir::ConstantKind::StringBytes(point.filename.as_bytes().to_vec()),
            },
            integer(point.line)?,
            integer(point.column)?,
        ]),
    })
}

impl Resolver<'_> {
    pub(crate) fn ast_source_location(&self, span: Span) -> Result<SourceSpan, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "source directive requires source module resolution")
        })?;
        Ok(SourceSpan {
            source: self.debug.source().unwrap_or(scope.source()),
            span,
        })
    }

    fn source_point_at(&self, location: SourceSpan) -> Result<SourcePoint<'_>, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::at_source(
                location,
                "source directive requires source module resolution",
            )
        })?;
        let record = scope.source_record(location.source).ok_or_else(|| {
            Diagnostic::at_source(location, "source origin record is not retained")
        })?;
        source_point(record, location)
    }

    pub(crate) fn source_location_at(
        &self,
        ty: TypeId,
        location: SourceSpan,
    ) -> Result<ValueExpr, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::at_source(
                location,
                "source directive requires source module resolution",
            )
        })?;
        scope.validate_caller_location_type(ty, self.types, location.span)?;
        let policy = self.target_layout.ok_or_else(|| {
            Diagnostic::at_source(
                location,
                "source location requires an explicit target layout",
            )
        })?;
        LayoutEngine::new(self.types, policy)
            .layout(ty)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        let record = scope.source_record(location.source).ok_or_else(|| {
            Diagnostic::at_source(location, "source origin record is not retained")
        })?;
        Ok(location_constant(record, location, ty, self.types)?.into_expression())
    }

    pub(crate) fn source_location_expression(&self, span: Span) -> Result<Expr, Diagnostic> {
        let ty = self.caller_location_type(span)?;
        Ok(Expr::Typed {
            ty,
            value: self.source_location_at(ty, self.ast_source_location(span)?)?,
        })
    }

    pub(crate) fn source_file_expression(&self, span: Span) -> Result<Expr, Diagnostic> {
        let point = self.source_point_at(self.ast_source_location(span)?)?;
        Ok(Expr::Typed {
            ty: self.types.string(),
            value: ValueExpr::StringBytes {
                ty: self.types.string(),
                bytes: point.filename.as_bytes().to_vec(),
            },
        })
    }

    pub(crate) fn source_filepath_expression(&self, span: Span) -> Result<Expr, Diagnostic> {
        let location = self.ast_source_location(span)?;
        let point = self.source_point_at(location)?;
        let directory = point.directory.ok_or_else(|| {
            Diagnostic::at_source(location, "#filepath requires a retained source directory")
        })?;
        Ok(Expr::Typed {
            ty: self.types.string(),
            value: ValueExpr::StringBytes {
                ty: self.types.string(),
                bytes: directory.as_bytes().to_vec(),
            },
        })
    }

    pub(crate) fn source_line_expression(&self, span: Span) -> Result<Expr, Diagnostic> {
        let location = self.ast_source_location(span)?;
        let point = self.source_point_at(location)?;
        Ok(Expr::Int(coordinate(point.line, location)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn coordinates_use_retained_identity_and_unicode_scalars() {
        let mut sources = SourceMap::default();
        let source = sources.insert("/exact/café.jai".into(), "first\r\né🦀 #location()".into());
        let offset = sources.get(source).unwrap().text().find('#').unwrap();
        let location = SourceSpan {
            source,
            span: Span::new(offset, offset + "#location()".len()),
        };
        let point = source_point(sources.get(source).unwrap(), location).unwrap();
        assert_eq!(point.filename, "/exact/café.jai");
        assert_eq!(point.directory, Some("/exact"));
        assert_eq!((point.line, point.column), (2, 4));

        let other = sources.insert("/other.jai".into(), "first\r\né🦀 #location()".into());
        assert!(source_point(sources.get(other).unwrap(), location).is_err());
        assert!(
            source_point(
                sources.get(source).unwrap(),
                SourceSpan {
                    source,
                    span: Span::new(8, 9),
                },
            )
            .is_err()
        );
    }

    #[test]
    fn source_directory_preserves_roots_and_does_not_invent_a_parent() {
        let mut sources = SourceMap::default();
        for (path, directory) in [("/main.jai", Some("/")), ("main.jai", None)] {
            let source = sources.insert(path.into(), "#filepath".into());
            let point = source_point(
                sources.get(source).unwrap(),
                SourceSpan {
                    source,
                    span: Span::new(0, "#filepath".len()),
                },
            )
            .unwrap();
            assert_eq!(point.directory, directory);
        }
    }
}
