# Changelog
## Version 0.2.0 <sup><i>WIP</i></sup>
### Additions
* Added `Option` support for modifiers
* Added `null` literal
* Added `span` to some types
* Added `ìnclude` statement

### Changes
* Modifiers now support unnamed arguments
* Some errors give better error messages
* Removed unused log dependency
* Improved error reporting
* TemplateKeys must implement the TemplateKey trait

### Deprecated
* `defaults::` attributes for `create_modifier` are deprecated and will be removed in a future version

### Internal
* Rewrote entire identifier code to support multipart modifiers
* Templates are now indexed by hashes

# Changelog
## Version 0.1.1
### Fixed
* Quote masking works as expected
* Fixed add sub mul div modifier when regex feature was not enabled

## Version 0.1.0
Initial release
