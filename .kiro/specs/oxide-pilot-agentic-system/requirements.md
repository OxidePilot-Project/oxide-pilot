# Requirements Document

## Introduction

Oxide Pilot es un sistema agéntico revolucionario que combina monitoreo de sistema, seguridad EDR, y asistencia conversacional por IA. El sistema está diseñado para evolucionar de una herramienta de monitoreo pasiva a un asistente proactivo que puede entender, aprender y actuar en el sistema del usuario. La arquitectura dual de agentes (Guardian y Copilot) proporciona tanto protección continua como asistencia bajo demanda, todo construido sobre una base de Rust de alto rendimiento con integración cloud mediante Google Vertex AI.

## Requirements

### Requirement 1: Sistema de Agentes Dual

**User Story:** Como usuario del sistema, quiero tener dos agentes especializados trabajando en mi equipo, para que pueda tener protección continua y asistencia conversacional cuando la necesite.

#### Acceptance Criteria

1. WHEN el sistema se inicia THEN el Guardian Agent SHALL comenzar a monitorear el sistema en segundo plano
2. WHEN el usuario dice la palabra clave "Hey Oxide" THEN el Copilot Agent SHALL activarse y mostrar la interfaz conversacional
3. WHEN ambos agentes están activos THEN SHALL compartir la misma base de conocimientos y memoria contextual
4. WHEN el Copilot Agent completa una tarea THEN SHALL actualizar la memoria compartida para futuras referencias
5. IF el Guardian Agent detecta una amenaza crítica THEN SHALL poder activar el Copilot Agent automáticamente

### Requirement 2: Monitoreo y Seguridad del Sistema (Guardian Agent)

**User Story:** Como administrador de sistema, quiero que el Guardian Agent monitoree continuamente mi sistema, para que pueda detectar amenazas y optimizar el rendimiento automáticamente.

#### Acceptance Criteria

1. WHEN el Guardian Agent está activo THEN SHALL monitorear procesos, uso de recursos y actividad de red en tiempo real
2. WHEN se detecta un proceso sospechoso THEN SHALL aplicar reglas de seguridad predefinidas y alertar al usuario
3. WHEN el uso de recursos excede umbrales críticos THEN SHALL tomar acciones de optimización automática
4. WHEN ocurre un evento de seguridad THEN SHALL registrar todos los detalles en la base de conocimientos
5. IF se detecta malware conocido THEN SHALL bloquear la ejecución y notificar inmediatamente

### Requirement 3: Interfaz Conversacional y Control por Voz (Copilot Agent)

**User Story:** Como usuario, quiero poder hablar naturalmente con mi sistema y que me entienda, para que pueda resolver problemas y ejecutar tareas de forma intuitiva.

#### Acceptance Criteria

1. WHEN el usuario pronuncia la palabra clave THEN el sistema SHALL activar la escucha de voz usando detección local
2. WHEN el usuario habla una consulta THEN SHALL transcribir el audio usando Google Speech-to-Text
3. WHEN se recibe una transcripción THEN SHALL procesar la consulta usando Vertex AI Gemini 1.5 Pro
4. WHEN el LLM genera una respuesta THEN SHALL sintetizar la voz usando Google Text-to-Speech
5. IF la consulta requiere información del sistema THEN SHALL consultar la memoria contextual antes de responder
6. WHEN el agente necesita más información THEN SHALL poder tomar capturas de pantalla automáticamente

### Requirement 4: Control Automatizado del Sistema (RPA)

**User Story:** Como usuario, quiero que el Copilot Agent pueda tomar acciones directas en mi sistema cuando sea necesario, para que pueda resolver problemas sin intervención manual.

#### Acceptance Criteria

1. WHEN el LLM determina que necesita ejecutar una acción THEN SHALL usar function calling para especificar la herramienta requerida
2. WHEN se requiere control del mouse THEN SHALL poder mover el cursor y hacer clic en coordenadas específicas
3. WHEN se requiere entrada de teclado THEN SHALL poder simular pulsaciones de teclas y combinaciones
4. WHEN se necesita información visual THEN SHALL poder tomar capturas de pantalla de regiones específicas
5. IF una acción requiere permisos elevados THEN SHALL solicitar confirmación explícita del usuario
6. WHEN se ejecuta una acción automatizada THEN SHALL registrar todos los pasos en la memoria para auditoría

### Requirement 5: Memoria Contextual y Aprendizaje (Cognee Integration)

**User Story:** Como usuario frecuente, quiero que el sistema recuerde nuestras interacciones pasadas y aprenda de mis patrones de uso, para que pueda brindar asistencia cada vez más personalizada.

#### Acceptance Criteria

1. WHEN ocurre cualquier evento del sistema THEN SHALL almacenar la información relevante en la base de conocimientos Cognee
2. WHEN el usuario hace una pregunta THEN SHALL consultar la memoria contextual para recuperar información relevante
3. WHEN se procesa una consulta THEN SHALL usar RAG (Retrieval-Augmented Generation) para enriquecer el contexto del LLM
4. WHEN se completa una interacción THEN SHALL actualizar el grafo de conocimiento con nuevas relaciones
5. IF se detectan patrones recurrentes THEN SHALL sugerir automatizaciones proactivas
6. WHEN el sistema se reinicia THEN SHALL mantener toda la memoria contextual persistente

### Requirement 6: Análisis Multimodal y Visual

**User Story:** Como usuario, quiero que el agente pueda "ver" mi pantalla y entender problemas visuales, para que pueda ayudarme con errores que aparecen en interfaces gráficas.

#### Acceptance Criteria

1. WHEN el usuario menciona un problema visual THEN el agente SHALL tomar una captura de pantalla automáticamente
2. WHEN se captura una imagen THEN SHALL enviarla junto con el contexto textual a Gemini 1.5 Pro
3. WHEN el LLM analiza una imagen THEN SHALL poder identificar elementos de UI, errores y problemas visuales
4. WHEN se detecta un error en pantalla THEN SHALL correlacionar con logs del sistema para diagnóstico completo
5. IF el usuario pregunta sobre algo específico en pantalla THEN SHALL poder analizar regiones específicas de la captura

### Requirement 7: Arquitectura Multi-Proveedor de IA y Seguridad

**User Story:** Como usuario empresarial, quiero poder elegir entre diferentes proveedores de IA y usar modelos locales cuando sea necesario, para que pueda tener control total sobre mis datos y costos.

#### Acceptance Criteria

1. WHEN se configura el sistema THEN SHALL permitir seleccionar entre múltiples proveedores de LLM (Google Vertex AI, OpenAI, Anthropic, Azure OpenAI)
2. WHEN se usa un proveedor externo THEN SHALL permitir configurar API keys personalizadas de forma segura
3. WHEN se requiere máxima privacidad THEN SHALL poder usar modelos locales (Ollama, LM Studio, etc.)
4. WHEN se procesa audio THEN SHALL soportar tanto servicios cloud como modelos locales de STT/TTS
5. WHEN se autentica un usuario THEN SHALL usar Firebase Authentication para gestión segura
6. WHEN se transmiten datos sensibles THEN SHALL usar encriptación end-to-end
7. IF se detecta actividad sospechosa THEN SHALL aplicar medidas de seguridad adicionales automáticamente
8. WHEN se almacenan datos localmente THEN SHALL encriptar toda la información sensible
9. WHEN se usan modelos locales THEN SHALL optimizar automáticamente según hardware disponible

### Requirement 8: Interfaz de Usuario Adaptativa

**User Story:** Como usuario, quiero una interfaz que se adapte a diferentes contextos de uso, para que pueda tener una experiencia fluida tanto en monitoreo pasivo como en interacción activa.

#### Acceptance Criteria

1. WHEN el sistema está en modo monitoreo THEN SHALL mostrar una interfaz compacta y no intrusiva
2. WHEN se activa el Copilot Agent THEN SHALL mostrar la animación del agente y la interfaz conversacional
3. WHEN el agente está procesando THEN SHALL mostrar estados visuales apropiados (pensando, escuchando, hablando)
4. WHEN se completa una interacción THEN SHALL poder minimizar automáticamente a modo compacto
5. IF el usuario prefiere modo siempre visible THEN SHALL mantener la interfaz expandida según configuración
6. WHEN ocurren alertas críticas THEN SHALL mostrar notificaciones prominentes sin interrumpir el flujo de trabajo

### Requirement 9: Configuración y Personalización

**User Story:** Como usuario avanzado, quiero poder configurar el comportamiento del sistema según mis necesidades específicas, para que se adapte a mi flujo de trabajo y preferencias.

#### Acceptance Criteria

1. WHEN el usuario accede a configuración THEN SHALL poder ajustar sensibilidad de detección de amenazas
2. WHEN se configuran preferencias THEN SHALL poder personalizar la palabra clave de activación
3. WHEN se establecen permisos THEN SHALL poder definir qué acciones automáticas están permitidas
4. WHEN se configura la voz THEN SHALL poder seleccionar diferentes voces y velocidades de síntesis
5. IF el usuario es desarrollador THEN SHALL poder acceder a logs detallados y métricas del sistema
6. WHEN se cambian configuraciones THEN SHALL aplicar los cambios sin requerir reinicio del sistema

### Requirement 10: Soporte para Modelos Locales y Edge Computing

**User Story:** Como usuario que valora la privacidad y el control, quiero poder ejecutar modelos de IA localmente en mi hardware, para que mis datos nunca salgan de mi equipo y pueda funcionar sin conexión a internet.

#### Acceptance Criteria

1. WHEN se configura modo local THEN SHALL soportar modelos LLM locales via Ollama, LM Studio o GGML
2. WHEN se usan modelos locales THEN SHALL detectar automáticamente hardware disponible (CPU, GPU, NPU)
3. WHEN se ejecuta inferencia local THEN SHALL optimizar el uso de recursos según capacidades del hardware
4. WHEN no hay conexión a internet THEN SHALL funcionar completamente con modelos locales
5. IF el hardware es limitado THEN SHALL sugerir modelos optimizados para el dispositivo específico
6. WHEN se cambia entre modelos THEN SHALL mantener consistencia en la experiencia del usuario
7. WHEN se usan modelos locales THEN SHALL permitir fine-tuning con datos del usuario
8. IF se requiere mayor capacidad THEN SHALL poder combinar modelos locales con cloud de forma híbrida

### Requirement 11: Rendimiento y Escalabilidad

**User Story:** Como usuario con un sistema de recursos limitados, quiero que Oxide Pilot sea eficiente y no impacte el rendimiento de mi equipo, para que pueda mantenerlo activo sin degradación del sistema.

#### Acceptance Criteria

1. WHEN el Guardian Agent está activo THEN SHALL usar menos del 5% de CPU en promedio
2. WHEN el sistema está en reposo THEN SHALL usar menos de 100MB de RAM
3. WHEN se procesa audio THEN SHALL mantener latencia menor a 500ms para detección de wake word
4. WHEN se ejecutan múltiples tareas THEN SHALL priorizar recursos según criticidad de la operación
5. IF los recursos del sistema son limitados THEN SHALL ajustar automáticamente la frecuencia de monitoreo
6. WHEN se detecta alta carga del sistema THEN SHALL reducir temporalmente las operaciones no críticas
7. WHEN se usan modelos locales THEN SHALL balancear automáticamente carga entre CPU y GPU
8. IF se detecta sobrecarga de memoria THEN SHALL optimizar el tamaño de contexto de los modelos