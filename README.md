# Tasks War Arena

El objetivo de este proyecto es armar una competencia de programación donde los
usuarios suban programas en WebAssembly que participen en el juego _Tasks War_.

Se trata de un juego de dos jugadores, los cuáles son controlados por los
programas subidos. Cada jugador arranca con una tarea y debe llamar a las
_syscalls_ disponibles para interactuar con la arena de juego. 

Al darse las condiciones de finalización del juego, el jugador que haya
conseguido más puntos será declarado ganador.

# Reglas del juego

COMPLETAR

# Paquetes

Este repositorio aloja un conjunto de paquetes que forman el ecosistema de
*tasks_war*.

* `tasks-war-bindings` esta librería expone _bindings_ para facilitar el llamado
  a las syscall del juego.

* `tasks-war-examples` son ejemplos de "jugadores" que se compilan a WASM para
  participar en el juego.

* `tasks-war-runtime` es el runtime que corre una competición y un jugador
  (programa WebAssembly compilado). Es una librería.

* `tasks-war-runtime-bin` es el ejecutable de línea de comandos para ejecutar la
  competición

* `tasks-war-visualizer` es la librería WASM para ejecutar el visualizador del
  juego en el navegador web. 

* `tasks-war-website` es el sitio web dónde se aloja el juego.



