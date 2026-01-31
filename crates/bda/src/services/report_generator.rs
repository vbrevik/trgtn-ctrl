// BDA Report Generator Service
// Purpose: Generate standardized BDA reports in various formats

use crate::features::bda::domain::{
    bda_report::BdaReport,
    report_template::GenerateReportRequest,
};
use super::formatters::{json, html, kml, pdf};

pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate report data structure (JSON format)
    pub fn generate_json(
        report: &BdaReport,
        request: &GenerateReportRequest,
    ) -> Result<serde_json::Value, String> {
        json::generate_json(report, request)
    }
    
    /// Generate HTML report
    pub fn generate_html(
        report: &BdaReport,
        request: &GenerateReportRequest,
    ) -> Result<String, String> {
        html::generate_html(report, request)
    }
    
    /// Generate KML for geospatial visualization
    pub fn generate_kml(
        report: &BdaReport,
        request: &GenerateReportRequest,
    ) -> Result<String, String> {
        kml::generate_kml(report, request)
    }
    
    /// Generate PDF report using printpdf
    /// Note: This creates a basic PDF. For production use, consider adding proper font support
    /// and more sophisticated layout using the Op-based API.
    pub fn generate_pdf(
        report: &BdaReport,
        request: &GenerateReportRequest,
    ) -> Result<Vec<u8>, String> {
        pdf::generate_pdf(report, request)
    }
}
