use aws_sdk_bedrockruntime::types::DocumentFormat;
use beetl::{
    completion::CompletionError,
    message::{DocumentMediaType, MimeType},
};

pub struct beetlDocumentMediaType(pub DocumentMediaType);

impl TryFrom<beetlDocumentMediaType> for DocumentFormat {
    type Error = CompletionError;

    fn try_from(value: beetlDocumentMediaType) -> Result<Self, Self::Error> {
        match value.0 {
            DocumentMediaType::PDF => Ok(DocumentFormat::Pdf),
            DocumentMediaType::TXT => Ok(DocumentFormat::Txt),
            DocumentMediaType::HTML => Ok(DocumentFormat::Html),
            DocumentMediaType::MARKDOWN => Ok(DocumentFormat::Md),
            DocumentMediaType::CSV => Ok(DocumentFormat::Csv),
            e => Err(CompletionError::ProviderError(format!(
                "Unsupported media type {}",
                e.to_mime_type()
            ))),
        }
    }
}

impl TryFrom<DocumentFormat> for beetlDocumentMediaType {
    type Error = CompletionError;

    fn try_from(value: DocumentFormat) -> Result<Self, Self::Error> {
        match value {
            DocumentFormat::Csv => Ok(beetlDocumentMediaType(DocumentMediaType::CSV)),
            DocumentFormat::Html => Ok(beetlDocumentMediaType(DocumentMediaType::HTML)),
            DocumentFormat::Md => Ok(beetlDocumentMediaType(DocumentMediaType::MARKDOWN)),
            DocumentFormat::Pdf => Ok(beetlDocumentMediaType(DocumentMediaType::PDF)),
            DocumentFormat::Txt => Ok(beetlDocumentMediaType(DocumentMediaType::TXT)),
            e => Err(CompletionError::ProviderError(format!(
                "Unsupported media type {}",
                e
            ))),
        }
    }
}
