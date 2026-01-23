# Image

Displays an image with loading + fallback placeholders.

## Example

```rust
use std::sync::Arc;
use gpui::Image;
use yororen_ui::component::image;

let img: Arc<Image> = yororen_ui::component::image_from_bytes(vec![]);
let view = image(img);
```
