# Sistema Solar Interactivo 🌌🚀

Este proyecto simula un sistema solar interactivo utilizando [Rust](https://www.rust-lang.org/) con la biblioteca [wgpu](https://github.com/gfx-rs/wgpu) para gráficos 3D. El usuario puede explorar el sistema solar, mover una nave espacial de manera manual y realizar teletransportes instantáneos entre planetas con un efecto animado. Además, los planetas orbitan automáticamente alrededor del sol, creando un ambiente dinámico y atractivo.

---

## Características

- 🌍 **Simulación del sistema solar**: Incluye una estrella central, planetas con colores y órbitas automáticas.
- 🚀 **Control de la nave espacial**: Mueve la nave manualmente o teletranspórtala entre planetas.
- ✨ **Warping instantáneo animado**: La nave se desvanece, se teletransporta al planeta seleccionado y reaparece con un efecto visual.
- 🌌 **Skybox estrellado**: Fondo de estrellas para mayor realismo.
- ⚙️ **Rendimiento optimizado**: Maneja múltiples objetos y animaciones de manera eficiente.

## Instalación

Sigue estos pasos para ejecutar el proyecto en tu máquina local:

1. **Clona este repositorio**:
   ```bash
   git clone https://github.com/Andyfer004/Space-Travel.git
   cd Space-Travel
   ```

2. **Instala Rust**:
   Si no tienes Rust instalado, puedes instalarlo con:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Ejecuta el proyecto**:
   ```bash
   cargo run
   ```

4. **¡Disfruta de tu sistema solar! 🌌**

## Controles

### Movimiento manual

| Tecla | Acción |
|-------|--------|
| `W` | Avanzar |
| `S` | Retroceder |
| `A` | Mover a la izquierda |
| `D` | Mover a la derecha |
| `Space` | Subir |
| `LShift` | Bajar |
| Flecha Izquierda | Rotar izquierda |
| Flecha Derecha | Rotar derecha |
| Flecha Arriba | Rotar hacia arriba |
| Flecha Abajo | Rotar hacia abajo |

### Warping instantáneo

| Tecla | Destino |
|-------|---------|
| `1` | Warp al Sol |
| `2` | Warp a Mercurio |
| `3` | Warp a Venus |
| `4` | Warp a la Tierra |
| `5` | Warp a Marte |
| `6` | Warp a Júpiter |
| `7` | Warp a Saturno |
| `8` | Warp a Urano |

## Video de demostración

Aquí puedes ver una demostración del sistema en funcionamiento:

   - **Enlace al video**: [https://youtu.be/dXPGvslMZzU](https://youtu.be/dXPGvslMZzU).

## Estructura del proyecto

```
📁 Raíz del proyecto
├── src/
│   ├── main.rs         # Código principal
│   ├── shaders.rs      # Shaders utilizados en el renderizado
│   ├── ...
├── assets/
│   ├── model3d.obj     # Modelo 3D de la nave
├── Cargo.toml          # Configuración de dependencias
├── README.md           # Este archivo
```

## Requisitos técnicos

* **Rust**: Última versión estable
* **GPU compatible con wgpu**: Asegúrate de que tu GPU soporte gráficos 3D modernos
* **Cargo**: Administrador de paquetes de Rust

## Créditos

Proyecto desarrollado como parte de un curso de gráficos computacionales utilizando Rust y wgpu.

¡Espero que disfruten explorando el sistema solar! 🌠
