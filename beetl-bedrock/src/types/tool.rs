use aws_sdk_bedrockruntime::types as aws_bedrock;

use beetl::{
    completion::CompletionError,
    message::{Text, ToolResultContent},
};
use serde_json::Value;

use super::{image::beetlImage, json::AwsDocument};

pub struct beetlToolResultContent(pub ToolResultContent);

impl TryFrom<beetlToolResultContent> for aws_bedrock::ToolResultContentBlock {
    type Error = CompletionError;

    fn try_from(value: beetlToolResultContent) -> Result<Self, Self::Error> {
        match value.0 {
            ToolResultContent::Text(text) => {
                Ok(aws_bedrock::ToolResultContentBlock::Text(text.text))
            }
            ToolResultContent::Image(image) => {
                let image = beetlImage(image).try_into()?;
                Ok(aws_bedrock::ToolResultContentBlock::Image(image))
            }
        }
    }
}

impl TryFrom<aws_bedrock::ToolResultContentBlock> for beetlToolResultContent {
    type Error = CompletionError;

    fn try_from(value: aws_bedrock::ToolResultContentBlock) -> Result<Self, Self::Error> {
        match value {
            aws_bedrock::ToolResultContentBlock::Image(image) => {
                let image: beetlImage = image.try_into()?;
                Ok(beetlToolResultContent(ToolResultContent::Image(image.0)))
            }
            aws_bedrock::ToolResultContentBlock::Json(document) => {
                let json: Value = AwsDocument(document).into();
                Ok(beetlToolResultContent(ToolResultContent::Text(Text {
                    text: json.to_string(),
                })))
            }
            aws_bedrock::ToolResultContentBlock::Text(text) => {
                Ok(beetlToolResultContent(ToolResultContent::Text(Text { text })))
            }
            _ => Err(CompletionError::ProviderError(
                "ToolResultContentBlock contains unsupported variant".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use aws_sdk_bedrockruntime::types as aws_bedrock;
    use base64::{prelude::BASE64_STANDARD, Engine};
    use beetl::{
        completion::CompletionError,
        message::{ContentFormat, Image, ImageMediaType, Text, ToolResultContent},
    };

    use crate::types::tool::beetlToolResultContent;

    #[test]
    fn beetl_tool_text_to_aws_tool() {
        let tool = beetlToolResultContent(ToolResultContent::Text(Text { text: "42".into() }));
        let aws_tool: Result<aws_bedrock::ToolResultContentBlock, _> = tool.try_into();
        assert_eq!(aws_tool.is_ok(), true);
        assert_eq!(
            String::from(aws_tool.unwrap().as_text().unwrap()),
            String::from("42")
        );
    }

    #[test]
    fn beetl_tool_image_to_aws_tool() {
        let image = Image {
            data: BASE64_STANDARD.encode("img_data"),
            format: Some(ContentFormat::Base64),
            media_type: Some(ImageMediaType::JPEG),
            detail: None,
        };
        let tool = beetlToolResultContent(ToolResultContent::Image(image));
        let aws_tool: Result<aws_bedrock::ToolResultContentBlock, _> = tool.try_into();
        assert_eq!(aws_tool.is_ok(), true);
        assert_eq!(aws_tool.unwrap().is_image(), true)
    }

    #[test]
    fn aws_tool_to_beetl_tool() {
        let aws_tool = aws_bedrock::ToolResultContentBlock::Text("txt".into());
        let tool: Result<beetlToolResultContent, _> = aws_tool.try_into();
        assert_eq!(tool.is_ok(), true);
        let tool = match tool.unwrap().0 {
            ToolResultContent::Text(text) => Ok(text),
            _ => Err("tool doesn't contain text"),
        };
        assert_eq!(tool.is_ok(), true);
        assert_eq!(tool.unwrap().text, String::from("txt"))
    }

    #[test]
    fn aws_tool_to_unsupported_beetl_tool() {
        let document_source =
            aws_bedrock::DocumentSource::Bytes(aws_smithy_types::Blob::new("document_data"));
        let aws_document = aws_bedrock::DocumentBlock::builder()
            .format(aws_bedrock::DocumentFormat::Pdf)
            .name("Document")
            .source(document_source)
            .build()
            .unwrap();
        let aws_tool = aws_bedrock::ToolResultContentBlock::Document(aws_document);
        let tool: Result<beetlToolResultContent, _> = aws_tool.try_into();
        assert_eq!(tool.is_ok(), false);
        assert_eq!(
            tool.err().unwrap().to_string(),
            CompletionError::ProviderError(
                "ToolResultContentBlock contains unsupported variant".into()
            )
            .to_string()
        )
    }
}
