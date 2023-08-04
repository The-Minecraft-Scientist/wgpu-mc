package dev.birb.wgpu.mixin.core;

import dev.birb.wgpu.rust.WgpuNative;
import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.util.Identifier;
import net.minecraft.util.registry.Registry;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.stream.Collectors;

@Mixin(Registry.class)
public class RegistryMixin {

    @Inject(method = "register(Lnet/minecraft/util/registry/Registry;Lnet/minecraft/util/Identifier;Ljava/lang/Object;)Ljava/lang/Object;", at = @At("RETURN"))
    private static void registryHook(Registry<?> registry, Identifier id, Object entry, CallbackInfoReturnable<Object> cir) {
        if (entry instanceof Block block) {
            String blockId = Registry.BLOCK.getId(block).toString();

            WgpuNative.registerBlock(blockId);

            for(BlockState state : block.getStateManager().getStates()) {
                String stateKey = state.getEntries().entrySet().stream().map(net.minecraft.state.State.PROPERTY_MAP_PRINTER).collect(Collectors.joining(","));
                WgpuNative.registerBlockState(state, blockId, stateKey);
            }
        }
    }

}
