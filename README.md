```
 _     _       _   
| |__ | | ___ | |_
| '_ \| |/ _ \| __|
| |_) | | (_) | |_
|_.__/|_|\___/ \__|

 USAGE 
blot <object> [--args]
object :: cube or uv_sphere
--texture :: path to input texture
--out :: path to output gif file
--resolution :: gif resolution (1 - 65535)
--interpolation :: nearest, bilinear or bicubic
--lat :: latitude steps (3 - 65535) for uv_sphere
--lng :: longitude steps (3 - 65535) for uv_sphere
```
**3D Quad-Based Software Renderer made with <3 in Rust**

<img src="./doc/uv_llq.gif" alt="6;3 UV Sphere" style="max-width: 30%; display: inline-block">
<img src="./doc/uv_lq.gif" alt="12;6 UV Sphere" style="max-width: 30%; display: inline-block">
<img src="./doc/uv.gif" alt="128;64 UV Sphere" style="max-width: 30%; display: inline-block">