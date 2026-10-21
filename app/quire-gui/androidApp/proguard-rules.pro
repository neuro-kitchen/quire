
# The engine calls the certificate verifier's Kotlin classes through JNI: R8 cannot see that.
-keep, includedescriptorclasses class org.rustls.platformverifier.** { *; }
# JNA and the generated engine bindings (reflection and JNI).
-keep class com.sun.jna.** { *; }
-keep class dev.neurokitchen.quire.ffi.** { *; }
-keep class dev.neurokitchen.quire.data.engine.AndroidTls { *; }
