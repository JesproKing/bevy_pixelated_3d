# bevy_pixelated_3d
A project to convert any 3d world into a game that looks like pixelart, with builtin outlines based on depth and normal textures.

![image](https://github.com/user-attachments/assets/85bb0905-e500-4382-9944-cf97908e8a08)

This project makes use of a 3d Camera, with the PixelCamera component. To add objects to the 3d world, just add PIXEL_PERFECT_LAYERS (or RenderLayers::layer(0)) as a component. The resolution of the screen can be set with the constants at the top of pixel_camera.rs (there may be some hardcoded in the shader, not certain.)

The current shader has some half-finished dither and quantization, but those are easy enough to remove by commenting out a single line.
![image](https://github.com/user-attachments/assets/046cd983-2956-46cb-92cf-0023af5940e1)

Good luck with your projects! I would love to see what you make with this, so feel free to ping me when you showcase it, or if you have any trouble understanding the code.
