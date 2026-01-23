# Avatar

Displays a user/avatar image.

## Example

```rust
use gpui::{Image, px};
use std::sync::Arc;
use yororen_ui::component::{avatar, AvatarShape};

let image: Option<Arc<Image>> = None;
let view = avatar(image).shape(AvatarShape::Circle);
```
