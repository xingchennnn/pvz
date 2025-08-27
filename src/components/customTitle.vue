<template>
    <div>
        {{ title }}
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

defineModel('customTitle')

const title = ref('')


</script>

<style scoped>

</style>