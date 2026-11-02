# Ray marching on the GPU

The same idea as [raymarch](../raymarch/README.md), with one ray per pixel in a WGSL fragment
shader. The Jai side is the standard WebGPU C API; the playground draws in its Render tab (Chrome
and Edge; other browsers skip it), and natively it opens a window.

## Map

- [main.jai](main.jai): Calls the section.
- [raymarch_gpu.jai](raymarch_gpu.jai): The WebGPU setup and the frame loop.
- [scene.jai](scene.jai): The WGSL shader, shared with `examples/webgpu-raymarch`.
- [browser_canvas.jai](browser_canvas.jai): The browser's canvas and input.

## Things to try

- Edit the WGSL shader in `scene.jai`: its `palette` function colors the scene.
