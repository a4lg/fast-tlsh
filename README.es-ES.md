# fast-tlsh: Librería de Fuzzy Hashing compatible con TLSH, rápida y escrita en Rust puro

[TLSH](https://tlsh.org/) significa Trendmicro Locality Sensitive Hash.
TLSH puede utilizarse para detectar archivos similares.

Con este crate, puedes generar / analizar / comparar LSHs (compatibles con TLSH).

Gracias a las optimizaciones compatibles con SIMD y su diseño de memoria, la comparación de dos LSHs es significativamente más rápida que en la implementación original. Incluso si desactivas el SIMD real (para prohibir cualquier código unsafe), emplea operaciones pseudo-SIMD y tablas adicionales para acelerar la comparación.

Además, acelera la generación de fuzzy hashes (~50% más rápido) mediante la optimización de tabla de "doble actualización" (double update).


## Características del Crate (Principales)

*   `alloc` y `std` (por defecto)  
    Este crate es compatible con `no_std` (desactivando ambas) y `alloc` y `std` están construidos sobre la implementación mínima de `no_std`. Estas características habilitan las implementaciones que dependen de `alloc` y `std`, respectivamente.
*   `easy-functions` (por defecto)  
    Proporciona funciones de alto nivel fáciles de usar.
*   `simd` (por defecto; **rápido pero unsafe**)  
    Este crate es unsafe por defecto (debido al uso de instrucciones SIMD). Sin embargo, puedes beneficiarte de otras optimizaciones incluso si lo desactivas.
*   `detect-features` (por defecto; **marginalmente lento pero conveniente**)  
    Esta característica depende de `std`.  
    Si la característica `simd` está habilitada y existe la posibilidad de alternar entre implementaciones SIMD y no-SIMD, activa las comprobaciones en tiempo de ejecución para cambiar la implementación dinámicamente.
*   `opt-default` (por defecto; Recomendado si no hay características predeterminadas habilitadas)  
    Este crate implementa varias optimizaciones que pueden ajustarse por separado. Si desactivas todas las características por defecto, todas estas optimizaciones se desactivan. Puedes habilitar esta característica para obtener el conjunto recomendado de optimizaciones *excepto* las basadas en SIMD real (que generalmente son unsafe).
*   `opt-embedded-default` (Desactiva las características por defecto si usas esta)  
    Por defecto, este crate está optimizado para entornos con abundante caché. Para dispositivos embebidos con memoria caché reducida, puedes usar esta característica para evitar la generación de tablas grandes. Esto hace que el código sea ligeramente más grande, pero actualmente reduce la huella de memoria estática en 128.25KiB.
*   `strict-parser`  
    Habilita el analizador estricto que impone una validez adicional. Esto está desactivado por defecto (porque no está implementado en la versión oficial), pero habilitarlo hará que el analizador sea más robusto.
*   `unsafe` (**marginalmente rápido pero unsafe**)  
    Otras características unsafe no relacionadas con SIMD están ocultas tras la característica `unsafe` (desactivada por defecto). Ten en cuenta que, normalmente, habilitar esto no acelerará el programa significativamente.
*   `unstable`  
    Esta característica habilita algunas funcionalidades específicas de Rust Nightly *excepto* el portable SIMD. Ten en cuenta que esta característica depende fuertemente de la versión de `rustc` y no debe considerarse estable (no esperes semántica compatible con SemVer).
*   `serde`  
    Habilita la integración con [Serde](https://serde.rs/) para serializar / deserializar fuzzy hashes.
*   `tests-slow`  
    Habilita las pruebas "lentas" (incluyendo pruebas de fuzzing).

Para todas las características (incluidas las de ajuste menor), consulta la documentación.
