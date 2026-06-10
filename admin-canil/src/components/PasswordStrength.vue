<template>
  <div class="password-strength" v-if="password.length > 0">
    <div class="strength-bar-container">
      <div 
        class="strength-bar" 
        :class="strengthClass" 
        :style="{ width: strengthPercentage + '%' }"
      ></div>
    </div>
    <p class="strength-message" :class="strengthClass">
      {{ strengthMessage }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  password: string
}>()

const evaluateStrength = (pw: string) => {
  let score = 0;
  
  if (pw.length >= 6) score += 1;
  if (pw.length >= 10) score += 1;
  if (/\d/.test(pw)) score += 1;
  if (/[^a-zA-Z0-9]/.test(pw)) score += 1;

  return score;
}

const strengthPercentage = computed(() => {
  const score = evaluateStrength(props.password)
  if (score === 0) return 15;
  if (score === 1) return 30;
  if (score === 2) return 50;
  if (score === 3) return 75;
  return 100;
})

const strengthClass = computed(() => {
  const score = evaluateStrength(props.password)
  if (score <= 1) return 'weak';
  if (score === 2) return 'medium-weak';
  if (score === 3) return 'medium';
  return 'strong';
})

const strengthMessage = computed(() => {
  const score = evaluateStrength(props.password)
  if (score <= 1) return "Muito fraca: Fraca como um pug subindo escadas 🐕";
  if (score === 2) return "Fraca: Inseguro como um filhote de chihuahua tremendo 🐕";
  if (score === 3) return "Razoável: Razoável como a paciência de um gato 🐈";
  return "Forte: Forte como a coragem de um pinscher irritado! 🐶⚡";
})
</script>

<style scoped>
.password-strength {
  margin-top: 0.5rem;
  margin-bottom: 1rem;
}

.strength-bar-container {
  height: 6px;
  background-color: #e5e7eb;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 0.5rem;
}

.strength-bar {
  height: 100%;
  transition: width 0.3s ease, background-color 0.3s ease;
}

.strength-bar.weak { background-color: #ef4444; }
.strength-bar.medium-weak { background-color: #f97316; }
.strength-bar.medium { background-color: #eab308; }
.strength-bar.strong { background-color: #22c55e; }

.strength-message {
  font-size: 0.85rem;
  margin: 0;
  font-weight: 600;
}

.strength-message.weak { color: #ef4444; }
.strength-message.medium-weak { color: #f97316; }
.strength-message.medium { color: #eab308; }
.strength-message.strong { color: #16a34a; }
</style>
