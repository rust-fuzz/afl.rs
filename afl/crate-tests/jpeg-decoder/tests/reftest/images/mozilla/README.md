# mozilla
The files in this directory were taken from https://hg.mozilla.org/mozilla-central/file/tip/image/test/reftest/jpeg

The following changes were made:
* `jpg-gray.png` and `jpg-srgb-icc.png` were converted from RGB to grayscale using `convert <input.png> -colorspace gray <output.png>` to match the pixel format of the corresponding JPEG files.
