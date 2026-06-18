:man_page: mongoac_api

mongoac_api
===========

Synopsis
--------

.. code-block:: c

   #ifdef MONGOAC_STATIC
   #define MONGOAC_API
   #else
   #define MONGOAC_API /* ... (see below) ... */
   #endif

Description
-----------

Control symbol visibility in the public API.

When linking against the shared library, ``MONGOAC_API`` expands to
platform-specific import directives. When ``MONGOAC_STATIC`` is defined, it
expands to nothing.
