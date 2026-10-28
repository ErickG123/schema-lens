use schema_lens_domain::*;
use std::fs;
use std::path::Path;

pub struct ReportExporter;

impl ReportExporter {
    pub fn export_json(
        result: &ComparisonRunResult,
        path: impl AsRef<Path>,
    ) -> Result<usize, String> {
        let json = serde_json::to_string_pretty(result)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&path, &json).map_err(|e| format!("Failed to write JSON report: {}", e))?;
        Ok(json.len())
    }

    pub fn export_csv(
        result: &ComparisonRunResult,
        path: impl AsRef<Path>,
    ) -> Result<usize, String> {
        let mut csv = String::from("id,severity,object_type,schema,object,category,value_source,value_target,explanation\n");
        for d in &result.differences {
            csv.push_str(&format!(
                "\"{}\",\"{:?}\",\"{:?}\",\"{}\",\"{}\",\"{:?}\",\"{}\",\"{}\",\"{}\"\n",
                d.id,
                d.severity,
                d.object_type,
                d.schema_name,
                d.object_name,
                d.category,
                d.value_source.as_deref().unwrap_or(""),
                d.value_target.as_deref().unwrap_or(""),
                d.explanation.replace('"', "\"\"")
            ));
        }
        fs::write(&path, &csv).map_err(|e| format!("Failed to write CSV report: {}", e))?;
        Ok(csv.len())
    }

    pub fn export_html(
        result: &ComparisonRunResult,
        path: impl AsRef<Path>,
    ) -> Result<usize, String> {
        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>SchemaLens Report - {} vs {}</title>
  <style>
    body {{ font-family: sans-serif; background: #0f172a; color: #f8fafc; padding: 24px; }}
    h1 {{ color: #60a5fa; }}
    table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
    th, td {{ border: 1px solid #334155; padding: 10px; text-align: left; font-size: 14px; }}
    th {{ background: #1e293b; }}
    tr:nth-child(even) {{ background: #131c2e; }}
  </style>
</head>
<body>
  <h1>Relatório de Comparação de Schemas</h1>
  <p><b>Origem:</b> {} | <b>Destino:</b> {} | <b>Gerado em:</b> {}</p>
  <p><b>Total de Diferenças:</b> {} | <b>Críticas:</b> {}</p>
  <table>
    <tr><th>Severidade</th><th>Schema</th><th>Objeto</th><th>Categoria</th><th>Origem</th><th>Destino</th><th>Explicação</th></tr>
"#,
            result.source_database,
            result.target_database,
            result.source_database,
            result.target_database,
            result.generated_at_utc,
            result.summary.total_differences,
            result.summary.critical_severity
        );

        for d in &result.differences {
            html.push_str(&format!(
                "    <tr><td>{:?}</td><td>{}</td><td>{}</td><td>{:?}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                d.severity,
                d.schema_name,
                d.object_name,
                d.category,
                d.value_source.as_deref().unwrap_or("<i>N/A</i>"),
                d.value_target.as_deref().unwrap_or("<i>N/A</i>"),
                d.explanation
            ));
        }

        html.push_str("  </table>\n</body>\n</html>");
        fs::write(&path, &html).map_err(|e| format!("Failed to write HTML report: {}", e))?;
        Ok(html.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_json_and_csv() {
        let run = ComparisonRunResult {
            run_id: "test-run".to_string(),
            source_database: "DB1".to_string(),
            target_database: "DB2".to_string(),
            summary: ComparisonSummary {
                total_differences: 1,
                missing_in_target: 1,
                missing_in_source: 0,
                property_mismatches: 0,
                definition_mismatches: 0,
                indeterminate: 0,
                critical_severity: 0,
                medium_severity: 1,
                low_severity: 0,
            },
            differences: vec![SchemaDifference {
                id: "d1".to_string(),
                object_type: SchemaObjectType::Table,
                schema_name: "dbo".to_string(),
                object_name: "T1".to_string(),
                category: DifferenceCategory::MissingInTarget,
                property_name: None,
                value_source: None,
                value_target: None,
                severity: DifferenceSeverity::Medium,
                confidence: ConfidenceStatus::High,
                explanation: "Missing table".to_string(),
                risk_assessment: None,
                can_generate_sql: true,
            }],
            generated_at_utc: "2026-10-27T20:00:00Z".to_string(),
        };

        let temp_dir = std::env::temp_dir();
        let json_path = temp_dir.join("schemalens_test.json");
        let csv_path = temp_dir.join("schemalens_test.csv");

        let json_bytes = ReportExporter::export_json(&run, &json_path).unwrap();
        assert!(json_bytes > 0);

        let csv_bytes = ReportExporter::export_csv(&run, &csv_path).unwrap();
        assert!(csv_bytes > 0);

        let _ = fs::remove_file(json_path);
        let _ = fs::remove_file(csv_path);
    }
}
