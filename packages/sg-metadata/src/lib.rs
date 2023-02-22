use cosmwasm_schema::cw_serde;

/// An attribute of the token as defined by the
/// [OpenSea metadata standard](https://docs.opensea.io/docs/metadata-standards#attributes).
#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

/// OpenSea metadata standard, used by Stargaze marketplace.
/// See [this link](https://docs.opensea.io/docs/metadata-standards) for details.
#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    /// This is the URL to the image of the item. Can be just about any type of image (including
    /// SVGs, which will be cached into PNGs by OpenSea), and can be
    /// [IPFS](https://github.com/ipfs/is-ipfs) URLs or paths. We recommend using a 350 x 350 image.
    pub image: Option<String>,
    /// Raw SVG image data, if you want to generate images on the fly (not recommended). Only use
    /// this if you're not including the `image` parameter.
    pub image_data: Option<String>,
    /// This is the URL that will appear below the asset's image on OpenSea and will allow users to
    /// leave OpenSea and view the item on your site.
    pub external_url: Option<String>,
    /// A human readable description of the item. Markdown is supported.
    pub description: Option<String>,
    /// Name of the item.
    pub name: Option<String>,
    /// These are the attributes for the item, which will show up on the OpenSea page for the item.
    pub attributes: Option<Vec<Trait>>,
    /// Background color of the item on OpenSea. Must be a six-character hexadecimal without a
    /// pre-pended #.
    pub background_color: Option<String>,
    /// A URL to a multi-media attachment for the item. The file extensions GLTF, GLB, WEBM, MP4,
    /// M4V, OGV, and OGG are supported, along with the audio-only extensions MP3, WAV, and OGA.
    ///
    /// Animation_url also supports HTML pages, allowing you to build rich experiences and
    /// interactive NFTs using JavaScript canvas, WebGL, and more. Scripts and relative paths within
    /// the HTML page are now supported. However, access to browser extensions is not supported.
    pub animation_url: Option<String>,
    /// A URL to a YouTube video.
    pub youtube_url: Option<String>,
}
