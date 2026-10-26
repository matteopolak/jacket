# Ray marching on the GPU

The same idea as [raymarch](../raymarch/README.md), with one ray per pixel in a WGSL fragment
shader. The Jai side is the standard WebGPU C API; the playground draws in its Render tab (Chrome
and Edge; other browsers skip it), and natively it opens a window.

## Map

- [main.jai](main.jai): Calls the section.
- [raymarch_gpu.jai](raymarch_gpu.jai): The WebGPU setup, the frame loop and the shader.
- [browser_canvas.jai](browser_canvas.jai): The browser's canvas and input.

## Things to try

- Edit the WGSL shader: its `palette` function colors the scene.
