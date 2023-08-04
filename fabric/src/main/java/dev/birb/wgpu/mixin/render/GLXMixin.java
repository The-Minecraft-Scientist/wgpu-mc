package dev.birb.wgpu.mixin.render;

import com.mojang.blaze3d.platform.GLX;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.util.function.LongSupplier;

@Mixin(GLX.class)
public class GLXMixin {

    /**
     * @author wgpu-mc
     * @reason replaced with wgpu equivalent
     */
    @Overwrite(remap = false)
    public static LongSupplier _initGlfw() {
        return System::nanoTime;
    }

}
