# Data Hunter Pro
### Agente Autónomo de Descubrimiento de Datos e Inteligencia Artificial

![Rust](https://img.shields.io/badge/Backend-Rust-black?style=flat&logo=rust)
![React](https://img.shields.io/badge/Frontend-React-blue?style=flat&logo=react)
![Docker](https://img.shields.io/badge/Deployment-Docker-2496ED?style=flat&logo=docker)
![OpenAI](https://img.shields.io/badge/AI-Powered-00A67E?style=flat&logo=openai)
![License](https://img.shields.io/badge/License-MIT-green)

**Data Hunter Pro** es una herramienta de alto rendimiento diseñada para rastrear, identificar y clasificar datasets de código abierto en la web. Combina la velocidad de **Rust** para el crawling web con la potencia de **OpenAI (GPT-4o-mini)** para analizar semánticamente el contenido de los recursos encontrados.

Cuenta con una interfaz moderna en **React** que permite la monitorización en tiempo real mediante **Server-Sent Events (SSE)**.

---

## ✨ Características Principales

*   **🚀 Motor de Alto Rendimiento:** Backend escrito en Rust utilizando `Tokio` y `Axum` para manejo asíncrono y concurrente de múltiples hilos de búsqueda.
*   **🧠 Análisis Semántico con IA:** Integración con OpenAI para analizar URLs y generar descripciones sintéticas y categorización automática (Gobierno, Academia, Open Data).
*   **🎯 Búsqueda Dirigida:** Capacidad de subir listas de temáticas personalizadas (CSV/TXT) para realizar búsquedas específicas usando *dorking* avanzado.
*   **📡 Monitorización en Tiempo Real:** Dashboard interactivo con logs en vivo, barra de progreso y tabla de resultados dinámica.
*   **📦 Todo en Uno:** Empaquetado en una sola imagen Docker (Multi-stage build) que sirve tanto la API como el Frontend estático.
*   **💾 Exportación de Datos:** Descarga inmediata de los hallazgos en formato CSV.

---

## 🛠️ Stack Tecnológico

### Backend (Rust)
*   **Framework:** Axum
*   **Runtime:** Tokio
*   **HTTP Client:** Reqwest
*   **Embedding:** RustEmbed (para servir el frontend)
*   **Utilidades:** Scraper, Regex, Serde

### Frontend (React + Vite)
*   **Estilos:** TailwindCSS
*   **Iconos:** Lucide React
*   **Comunicación:** EventSource (SSE)

### Infraestructura
*   **Containerización:** Docker (Debian Slim)

---

## 📋 Prerrequisitos

*   **Docker** instalado en tu sistema.
*   Una **API Key de OpenAI** (necesaria para la función de análisis inteligente, aunque el crawler funciona en modo básico sin ella).

---

## 🚀 Instalación y Despliegue

### Opción 1: Docker (Recomendada)

1.  **Clonar el repositorio:**
    ```bash
    git clone https://github.com/tu-usuario/data-hunter-pro.git
    cd data-hunter-pro
    ```

2.  **Construir la imagen:**
    ```bash
    docker build -t data-hunter .
    ```

3.  **Ejecutar el contenedor:**
    Debes pasar tus variables de entorno, especialmente la `OPENAI_API_KEY`.
    ```bash
    docker run -d -p 3000:3000 \
      -e OPENAI_API_KEY="tu-api-key-aqui" \
      --name hunter-instance \
      data-hunter
    ```

4.  **Acceder:** Abre tu navegador en `http://localhost:3000`.

### Opción 2: Desarrollo Local (Manual)

**Backend:**
```bash
# Necesitas Rust instalado
cargo run
```

**Frontend:**
```bash
# En otra terminal, dentro de la carpeta /web
cd web
npm install
npm run dev
```

---

## 📖 Guía de Uso

1.  **Configuración del Objetivo:**
    *   En el panel izquierdo, establece el número de **Fuentes Objetivo** (ej. 50 datasets).
2.  **Búsqueda Temática (Opcional):**
    *   Si deseas buscar algo específico (ej. "Datos climáticos", "Finanzas 2024"), crea un archivo `.txt` o `.csv` con una temática por línea.
    *   Arrastra el archivo al área de **"Temáticas CSV"**.
3.  **Iniciar Caza:**
    *   Presiona el botón **INICIAR**.
    *   Verás los logs en tiempo real en la "Agéntic Console".
4.  **Resultados:**
    *   La tabla principal se llenará con los enlaces encontrados, la categoría detectada y la descripción generada por la IA.
5.  **Exportar:**
    *   Haz clic en "EXPORTAR CSV" para descargar tus resultados.

---

## ⚙️ Variables de Entorno

El sistema utiliza un archivo `.env` o variables de entorno del sistema. Las principales son:

| Variable | Descripción | Valor por defecto |
| :--- | :--- | :--- |
| `PORT` | Puerto de escucha del servidor | `3000` |
| `OPENAI_API_KEY` | Clave API para análisis inteligente | (Requerido para IA) |
| `AI_MODEL` | Modelo de OpenAI a utilizar | `gpt-4o-mini` |
| `RUST_LOG` | Nivel de log del backend | `info` |

---

## 📂 Estructura del Proyecto

```
.
├── Cargo.toml          # Dependencias de Rust
├── Dockerfile          # Construcción Multi-stage
├── src/
│   └── main.rs         # Lógica del servidor, crawler y IA
└── web/                # Frontend React
    ├── index.html
    ├── src/
    │   └── main.jsx    # UI Lógica
    ├── package.json
    └── vite.config.js
```

---

## 🛡️ Aviso Legal

Esta herramienta está diseñada con fines educativos y de investigación para la localización de datos abiertos (*Open Data*). El usuario es responsable de asegurar que el uso del crawler cumpla con los términos de servicio de los sitios web visitados y las regulaciones locales sobre scraping.

---

## 🤝 Contribución

¡Las contribuciones son bienvenidas! Si tienes ideas para mejorar el algoritmo de búsqueda o la interfaz:

1.  Haz un Fork del proyecto.
2.  Crea tu rama de características (`git checkout -b feature/AmazingFeature`).
3.  Haz Commit de tus cambios (`git commit -m 'Add some AmazingFeature'`).
4.  Push a la rama (`git push origin feature/AmazingFeature`).
5.  Abre un Pull Request.

---

Hecho con ❤️ y 🦀 (Rust).