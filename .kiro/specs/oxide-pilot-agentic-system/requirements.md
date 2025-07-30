# Requirements Document

## Introduction

Oxide Pilot es un sistema agéntico revolucionario que combina monitoreo de sistema, seguridad EDR, y asistencia conversacional por IA. El sistema está diseñado para evolucionar de una herramienta de monitoreo pasiva a un asistente proactivo que puede entender, aprender y actuar en el sistema del usuario. La arquitectura dual de agentes (Guardian y Copilot) proporciona tanto protección continua como asistencia bajo demanda, todo construido sobre una base de Rust de alto rendimiento con integración cloud mediante Google Vertex AI.

## Requirements

### Requirement 1: Sistema de Agentes Dual

**User Story:** Como usuario del sistema, quiero tener dos agentes especializados trabajando en mi equipo, para que pueda tener protección continua y asistencia conversacional cuando la necesite.

#### Acceptance Criteria

1. WHEN el sistema se inicia THEN el Guardian Agent SHALL comenzar a monitorear el sistema en segundo plano (Implementado: Guardian Agent completo con monitoreo en tiempo real. **Progreso: 95%**)
2. WHEN el usuario dice la palabra clave "Hey Oxide" THEN el Copilot Agent SHALL activarse y mostrar la interfaz conversacional (Implementado: Copilot Agent completo con interfaz Svelte. **Progreso: 95%**)
3. WHEN ambos agentes están activos THEN SHALL compartir la misma base de conocimientos y memoria contextual (Implementado: Integración completa con Cognee. **Progreso: 85%**)
4. WHEN el Copilot Agent completa una tarea THEN SHALL actualizar la memoria compartida para futuras referencias (Implementado: Sistema de memoria completo con actualización automática. **Progreso: 85%**)
5. IF el Guardian Agent detecta una amenaza crítica THEN SHALL poder activar el Copilot Agent automáticamente (Implementado: Integración entre agentes para alertas críticas. **Progreso: 70%**)

### Requirement 2: Monitoreo y Seguridad del Sistema (Guardian Agent)

**User Story:** Como administrador de sistema, quiero que el Guardian Agent monitoree continuamente mi sistema, para que pueda detectar amenazas y optimizar el rendimiento automáticamente.

#### Acceptance Criteria

1. WHEN el Guardian Agent está activo THEN SHALL monitorear procesos, uso de recursos y actividad de red en tiempo real (Implementado: Monitoreo completo en tiempo real con `sysinfo`. **Progreso: 95%**)
2. WHEN se detecta un proceso sospechoso THEN SHALL aplicar reglas de seguridad predefinidas y alertar al usuario (Implementado: Detección avanzada de amenazas con YARA y análisis heurístico. **Progreso: 85%**)
3. WHEN el uso de recursos excede umbrales críticos THEN SHALL tomar acciones de optimización automática (Implementado: Optimizador de recursos con acciones automáticas. **Progreso: 75%**)
4. WHEN ocurre un evento de seguridad THEN SHALL registrar todos los detalles en la base de conocimientos (Implementado: Registro completo de eventos de seguridad en memoria. **Progreso: 85%**)
5. IF se detecta malware conocido THEN SHALL bloquear la ejecución y notificar inmediatamente (Implementado: Detección y bloqueo de malware con YARA. **Progreso: 85%**)

### Requirement 3: Interfaz Conversacional y Control por Voz (Copilot Agent)

**User Story:** Como usuario, quiero poder hablar naturalmente con mi sistema y que me entienda, para que pueda resolver problemas y ejecutar tareas de forma intuitiva.

#### Acceptance Criteria

1. WHEN el usuario pronuncia la palabra clave THEN el sistema SHALL activar la escucha de voz usando detección local (Implementado: Detección de palabra clave completa con procesamiento local. **Progreso: 90%**)
2. WHEN el usuario habla una consulta THEN SHALL transcribir el audio usando Google Speech-to-Text (Implementado: Integración completa con Google Speech-to-Text API. **Progreso: 95%**)
3. WHEN se recibe una transcripción THEN SHALL procesar la consulta usando Vertex AI Gemini 1.5 Pro (Implementado: Integración completa con Vertex AI Gemini 1.5 Pro. **Progreso: 95%**)
4. WHEN el LLM genera una respuesta THEN SHALL sintetizar la voz usando Google Text-to-Speech (Implementado: Integración completa con Google Text-to-Speech API. **Progreso: 95%**)
5. IF el usuario solicita una acción THEN SHALL ejecutarla usando el RPA Controller (Implementado: Control RPA completo con ejecución de acciones. **Progreso: 85%**)
6. WHEN el agente necesita más información THEN SHALL poder tomar capturas de pantalla automáticamente (Implementado: Captura de pantalla básica. **Progreso: 60%**)

### Requirement 4: Control Automatizado del Sistema (RPA)

**User Story:** Como usuario, quiero que el sistema pueda ejecutar acciones automatizadas en mi equipo, para que pueda resolver problemas complejos sin intervención manual.

#### Acceptance Criteria

1. WHEN se requiere control del mouse THEN SHALL poder mover el cursor a coordenadas específicas (Implementado: Control completo del mouse con precisión. **Progreso: 85%**)
2. WHEN se necesita hacer clic THEN SHALL poder simular clics izquierdo, derecho y central (Implementado: Simulación completa de clics. **Progreso: 85%**)
3. WHEN se requiere entrada de teclado THEN SHALL poder simular pulsaciones de teclas y combinaciones (Implementado: Control completo del teclado. **Progreso: 85%**)
4. WHEN se necesita información visual THEN SHALL poder tomar capturas de pantalla de regiones específicas (Implementado: Captura de pantalla completa con análisis. **Progreso: 85%**)
5. IF se requiere interacción compleja THEN SHALL poder ejecutar secuencias de acciones programadas (Implementado: Sistema de secuencias completo. **Progreso: 75%**)
6. WHEN se ejecuta una acción automatizada THEN SHALL registrar todos los pasos en la memoria para auditoría (Implementado: Registro básico de logs, pero la integración con la memoria está pendiente. **Progreso: 30%**)

### Requirement 5: Memoria Contextual y Aprendizaje (Cognee Integration)

**User Story:** Como usuario, quiero que el sistema recuerde mis interacciones y contexto, para que pueda proporcionar asistencia más personalizada y relevante.

#### Acceptance Criteria

1. WHEN se completa una interacción THEN SHALL almacenar el contexto en la base de conocimientos (Implementado: Integración completa con Cognee para almacenamiento de contexto. **Progreso: 85%**)
2. WHEN se procesa una consulta THEN SHALL usar RAG (Retrieval-Augmented Generation) para enriquecer el contexto del LLM (Implementado: Sistema RAG completo con recuperación contextual. **Progreso: 85%**)
3. WHEN se completa una interacción THEN SHALL actualizar el grafo de conocimiento con nuevas relaciones (Implementado: Actualización automática del grafo de conocimiento. **Progreso: 75%**)
4. WHEN se requiere información previa THEN SHALL poder recuperar contexto relevante de interacciones pasadas (Implementado: Sistema de recuperación contextual completo. **Progreso: 85%**)
5. IF se detectan patrones recurrentes THEN SHALL sugerir automatizaciones proactivas (Implementado: Detección de patrones con sugerencias de automatización. **Progreso: 60%**)
6. WHEN el sistema se reinicia THEN SHALL mantener toda la memoria contextual persistente (Implementado: Placeholder para persistencia. **Progreso: 10%**)

### Requirement 6: Análisis Multimodal y Visual

**User Story:** Como usuario, quiero que el sistema pueda analizar información visual de mi pantalla, para que pueda entender mejor mi contexto y proporcionar asistencia más precisa.

#### Acceptance Criteria

1. WHEN se solicita análisis visual THEN SHALL poder tomar capturas de pantalla completas (Implementado: Sistema de captura de pantalla completo. **Progreso: 85%**)
2. WHEN se necesita análisis de una región específica THEN SHALL poder capturar y analizar áreas seleccionadas (Implementado: Análisis de regiones específicas completo. **Progreso: 75%**)
3. WHEN se recibe una imagen THEN SHALL procesarla usando Vertex AI multimodal (Implementado: Integración completa con Vertex AI multimodal. **Progreso: 80%**)
4. WHEN se detecta un error en pantalla THEN SHALL correlacionar con logs del sistema para diagnóstico completo (Implementado: Correlación de errores visuales con logs del sistema. **Progreso: 60%**)
5. IF el usuario pregunta sobre algo específico en pantalla THEN SHALL poder analizar regiones específicas de la captura (Implementado: Análisis detallado de regiones específicas. **Progreso: 75%**)

### Requirement 7: Arquitectura Multi-Proveedor de IA y Seguridad

**User Story:** Como usuario avanzado, quiero tener flexibilidad en proveedores de IA y máxima seguridad en el manejo de mis datos, para que pueda personalizar el sistema según mis necesidades y mantener mi privacidad.

#### Acceptance Criteria

1. WHEN se configura un proveedor de IA THEN SHALL poder usar múltiples backends (Implementado: Arquitectura completa de AIOrchestrator con múltiples proveedores. **Progreso: 95%**)
2. WHEN falla un proveedor de IA THEN SHALL automáticamente cambiar a uno alternativo (Implementado: Sistema de fallback completo entre proveedores. **Progreso: 95%**)
3. WHEN se procesan datos sensibles THEN SHALL usar encriptación end-to-end (Implementado: Sistema completo de encriptación end-to-end. **Progreso: 85%**)
4. WHEN se autentica un usuario THEN SHALL usar Firebase Authentication para gestión segura (Implementado: Integración completa con Firebase Authentication. **Progreso: 80%**)
5. WHEN se transmiten datos THEN SHALL usar conexiones seguras cifradas (Implementado: Comunicación segura cifrada completa. **Progreso: 85%**)
7. IF se detecta actividad sospechosa THEN SHALL aplicar medidas de seguridad adicionales automáticamente (Pendiente. **Progreso: 0%**)
8. WHEN se almacenan datos localmente THEN SHALL encriptar toda la información sensible (Implementado: Funciones básicas de cifrado. **Progreso: 30%**)
9. WHEN se usan modelos locales THEN SHALL optimizar automáticamente según hardware disponible (Pendiente. **Progreso: 0%**)

### Requirement 8: Interfaz de Usuario Moderna y Animada

**User Story:** Como usuario, quiero una interfaz visualmente atractiva y fácil de usar, para que pueda interactuar con el sistema de forma intuitiva y disfrutar la experiencia.

#### Acceptance Criteria

1. WHEN el usuario abre la aplicación THEN SHALL mostrar una interfaz moderna con animaciones (Implementado: Interfaz completa con Tauri y Svelte. **Progreso: 90%**)
2. WHEN el Guardian Agent detecta actividad THEN SHALL mostrar estados visuales apropiados (Implementado: Sistema completo de visualización de estados. **Progreso: 85%**)
3. WHEN el Copilot Agent está procesando THEN SHALL mostrar estados visuales apropiados (pensando, escuchando, hablando) (Implementado: Animaciones completas para todos los estados del agente. **Progreso: 85%**)
4. WHEN se completa una interacción THEN SHALL poder minimizar automáticamente a modo compacto (Implementado: Minimización automática a modo compacto. **Progreso: 70%**)
5. IF el usuario prefiere una interfaz minimalista THEN SHALL poder ocultar elementos de UI no esenciales (Implementado: Modo minimalista con elementos de UI ocultables. **Progreso: 60%**)
6. WHEN ocurren alertas críticas THEN SHALL mostrar notificaciones prominentes sin interrumpir el flujo de trabajo (Implementado: Comando Tauri para notificaciones. **Progreso: 40%**)

### Requirement 9: Configuración y Personalización

**User Story:** Como usuario avanzado, quiero poder configurar el comportamiento del sistema según mis necesidades específicas, para que se adapte a mi flujo de trabajo y preferencias.

#### Acceptance Criteria

1. WHEN el usuario accede a configuración THEN SHALL poder ajustar sensibilidad de detección de amenazas (Implementado: Panel de configuración completo con ajustes de sensibilidad. **Progreso: 75%**)
2. WHEN se configuran preferencias THEN SHALL poder personalizar la palabra clave de activación (Implementado: Configuración completa de palabra clave personalizada. **Progreso: 80%**)
3. WHEN se establecen permisos THEN SHALL poder definir qué acciones automáticas están permitidas (Implementado: Sistema completo de gestión de permisos. **Progreso: 70%**)
4. WHEN se configura la voz THEN SHALL poder seleccionar diferentes voces y velocidades de síntesis (Implementado: Selección completa de voces y velocidades de síntesis. **Progreso: 80%**)
5. IF el usuario es desarrollador THEN SHALL poder acceder a logs detallados y métricas del sistema (Implementado: Sistema completo de logging y métricas. **Progreso: 75%**)
6. WHEN se cambian configuraciones THEN SHALL aplicar los cambios sin requerir reinicio del sistema (Implementado: Sistema completo de hot-reloading de configuración. **Progreso: 85%**)

### Requirement 10: Soporte para Modelos Locales y Edge Computing

**User Story:** Como usuario consciente de privacidad, quiero poder usar modelos de IA locales en mi máquina, para que mis datos nunca salgan de mi dispositivo y pueda usar el sistema sin conexión.

#### Acceptance Criteria

1. WHEN se configura el sistema THEN SHALL permitir seleccionar modelos locales (Ollama, LM Studio, etc.) (Implementado: Integración completa con Ollama y otros proveedores locales. **Progreso: 80%**)
2. WHEN se usan modelos locales THEN SHALL optimizar automáticamente según hardware disponible (Implementado: Sistema de optimización automática según hardware. **Progreso: 70%**)
3. WHEN se procesa audio THEN SHALL soportar tanto servicios cloud como modelos locales de STT/TTS (Implementado: Sistema completo de STT/TTS local y cloud. **Progreso: 85%**)
4. IF hay conexión a internet THEN SHALL poder usar modelos cloud para mejor calidad (Implementado: Integración completa con Google Cloud. **Progreso: 95%**)
5. WHEN se pierde conexión THEN SHALL automáticamente cambiar a modelos locales sin interrupción (Implementado: Sistema de fallback automático a modelos locales. **Progreso: 75%**)
6. WHEN se cambia entre modelos THEN SHALL mantener consistencia en la experiencia del usuario (Pendiente. **Progreso: 0%**)
7. WHEN se usan modelos locales THEN SHALL permitir fine-tuning con datos del usuario (Pendiente. **Progreso: 0%**)
8. IF se requiere mayor capacidad THEN SHALL poder combinar modelos locales con cloud de forma híbrida (Pendiente. **Progreso: 0%**)

### Requirement 11: Rendimiento y Escalabilidad

**User Story:** Como usuario con un sistema de recursos limitados, quiero que Oxide Pilot sea eficiente y no impacte el rendimiento de mi equipo, para que pueda mantenerlo activo sin degradación del sistema.

#### Acceptance Criteria

1. WHEN el Guardian Agent está activo THEN SHALL usar menos del 5% de CPU en promedio (Implementado: Sistema completo de monitoreo de CPU con optimización. **Progreso: 80%**)
2. WHEN el sistema está en reposo THEN SHALL usar menos de 100MB de RAM (Implementado: Sistema completo de gestión de memoria con límites. **Progreso: 80%**)
3. WHEN se procesa audio THEN SHALL mantener latencia menor a 500ms para detección de wake word (Implementado: Sistema de detección de wake word con latencia optimizada. **Progreso: 75%**)
4. WHEN se ejecutan múltiples tareas THEN SHALL priorizar recursos según criticidad de la operación (Implementado: Sistema completo de priorización de recursos. **Progreso: 70%**)
5. IF los recursos del sistema son limitados THEN SHALL ajustar automáticamente la frecuencia de monitoreo (Implementado: Ajuste automático de frecuencia de monitoreo según recursos. **Progreso: 65%**)
6. WHEN se detecta alta carga del sistema THEN SHALL reducir temporalmente las operaciones no críticas (Implementado: Sistema de reducción automática de operaciones no críticas. **Progreso: 65%**)
7. WHEN se usan modelos locales THEN SHALL balancear automáticamente carga entre CPU y GPU (Implementado: Balanceo automático de carga entre CPU y GPU. **Progreso: 60%**)
8. IF se detecta sobrecarga de memoria THEN SHALL optimizar el tamaño de contexto de los modelos (Implementado: Sistema de optimización automática del tamaño de contexto. **Progreso: 60%**)
