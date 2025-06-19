use aws_sdk_bedrockruntime::types as aws_bedrock;
use beetl::{
    completion::CompletionError,
    message::{ContentFormat, Document},
};

pub(crate) use crate::types::media_types::beetlDocumentMediaType;
use base64::{prelude::BASE64_STANDARD, Engine};
use uuid::Uuid;

#[derive(Clone)]
pub struct beetlDocument(pub Document);

impl TryFrom<beetlDocument> for aws_bedrock::DocumentBlock {
    type Error = CompletionError;

    fn try_from(value: beetlDocument) -> Result<Self, Self::Error> {
        let document_media_type = value
            .0
            .media_type
            .map(|doc| beetlDocumentMediaType(doc).try_into());

        let document_media_type = match document_media_type {
            Some(Ok(document_format)) => Ok(Some(document_format)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }?;

        let document_data = match value.0.format {
            Some(ContentFormat::Base64) => BASE64_STANDARD
                .decode(value.0.data)
                .map_err(|e| CompletionError::ProviderError(e.to_string()))?,
            _ => value.0.data.as_bytes().to_vec(),
        };

        let data = aws_smithy_types::Blob::new(document_data);
        let document_source = aws_bedrock::DocumentSource::Bytes(data);

        let random_string = Uuid::new_v4().simple().to_string();
        let document_name = format!("document-{}", random_string);
        let result = aws_bedrock::DocumentBlock::builder()
            .source(document_source)
            .name(document_name)
            .set_format(document_media_type)
            .build()
            .map_err(|e| CompletionError::ProviderError(e.to_string()))?;
        Ok(result)
    }
}

impl TryFrom<aws_bedrock::DocumentBlock> for beetlDocument {
    type Error = CompletionError;

    fn try_from(value: aws_bedrock::DocumentBlock) -> Result<Self, Self::Error> {
        let media_type: beetlDocumentMediaType = value.format.try_into()?;
        let media_type = media_type.0;

        let data = match value.source {
            Some(aws_bedrock::DocumentSource::Bytes(blob)) => {
                let encoded_data = BASE64_STANDARD.encode(blob.into_inner());
                Ok(encoded_data)
            }
            _ => Err(CompletionError::ProviderError(
                "Document source is missing".into(),
            )),
        }?;

        Ok(beetlDocument(Document {
            data,
            format: Some(ContentFormat::Base64),
            media_type: Some(media_type),
        }))
    }
}

#[cfg(test)]
mod tests {
    use aws_sdk_bedrockruntime::types as aws_bedrock;
    use base64::{prelude::BASE64_STANDARD, Engine};
    use beetl::{
        completion::CompletionError,
        message::{ContentFormat, Document, DocumentMediaType},
    };

    use crate::types::document::beetlDocument;

    #[test]
    fn test_document_to_aws_document() {
        let beetl_document = beetlDocument(Document {
            data: "data".into(),
            format: Some(ContentFormat::String),
            media_type: Some(DocumentMediaType::PDF),
        });
        let aws_document: Result<aws_bedrock::DocumentBlock, _> = beetl_document.clone().try_into();
        assert_eq!(aws_document.is_ok(), true);
        let aws_document = aws_document.unwrap();
        assert_eq!(aws_document.format, aws_bedrock::DocumentFormat::Pdf);
        let document_data = beetl_document.0.data.as_bytes().to_vec();
        let aws_document_bytes = aws_document
            .source()
            .unwrap()
            .as_bytes()
            .unwrap()
            .as_ref()
            .to_owned();

        let doc_name = aws_document.name;
        assert!(doc_name.starts_with("document-"));
        assert_eq!(aws_document_bytes, document_data)
    }

    #[test]
    fn test_base64_document_to_aws_document() {
        let beetl_document = beetlDocument(Document {
            data: "data".into(),
            format: Some(ContentFormat::Base64),
            media_type: Some(DocumentMediaType::PDF),
        });
        let aws_document: aws_bedrock::DocumentBlock = beetl_document.clone().try_into().unwrap();
        let document_data = BASE64_STANDARD.decode(beetl_document.0.data).unwrap();
        let aws_document_bytes = aws_document
            .source()
            .unwrap()
            .as_bytes()
            .unwrap()
            .as_ref()
            .to_owned();
        assert_eq!(aws_document_bytes, document_data)
    }

    #[test]
    fn test_unsupported_document_to_aws_document() {
        let beetl_document = beetlDocument(Document {
            data: "data".into(),
            format: Some(ContentFormat::String),
            media_type: Some(DocumentMediaType::Javascript),
        });
        let aws_document: Result<aws_bedrock::DocumentBlock, _> = beetl_document.clone().try_into();
        assert_eq!(
            aws_document.err().unwrap().to_string(),
            CompletionError::ProviderError(
                "Unsupported media type application/x-javascript".into()
            )
            .to_string()
        )
    }

    #[test]
    fn test_aws_document_to_beetl_document() {
        let data = aws_smithy_types::Blob::new("document_data");
        let document_source = aws_bedrock::DocumentSource::Bytes(data);
        let aws_document = aws_bedrock::DocumentBlock::builder()
            .format(aws_bedrock::DocumentFormat::Pdf)
            .name("Document")
            .source(document_source)
            .build()
            .unwrap();
        let beetl_document: Result<beetlDocument, _> = aws_document.clone().try_into();
        assert_eq!(beetl_document.is_ok(), true);
        let beetl_document = beetl_document.unwrap().0;
        assert_eq!(beetl_document.media_type.unwrap(), DocumentMediaType::PDF)
    }

    #[test]
    fn test_unsupported_aws_document_to_beetl_document() {
        let data = aws_smithy_types::Blob::new("document_data");
        let document_source = aws_bedrock::DocumentSource::Bytes(data);
        let aws_document = aws_bedrock::DocumentBlock::builder()
            .format(aws_bedrock::DocumentFormat::Xlsx)
            .name("Document")
            .source(document_source)
            .build()
            .unwrap();
        let beetl_document: Result<beetlDocument, _> = aws_document.clone().try_into();
        assert_eq!(beetl_document.is_ok(), false);
        assert_eq!(
            beetl_document.err().unwrap().to_string(),
            CompletionError::ProviderError("Unsupported media type xlsx".into()).to_string()
        )
    }
}
