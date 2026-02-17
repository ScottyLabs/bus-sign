<script lang="ts">
  import BusTimeEntry from './lib/BusTimeEntry.svelte';
</script>

<main>
  <div class="container" style="justify-content: start; align-items: flex-start">
      <div class="stack left">
      <BusTimeEntry route={'61C'} location={'Downtown'} time={2} busNumber={6767}/>
      <BusTimeEntry route={'28X'} location={'Airporto'} time={11} busNumber={1234}/>
      <BusTimeEntry route={'61D'} location={'Oakland'} time={13} busNumber={8888}/>
      <BusTimeEntry route={'61A'} location={'CMU'} time={25} busNumber={5032}/>
    </div>
      <div class="stack right">
      <BusTimeEntry route={'28X'} location={'Airporto'} time={11} busNumber={1234}/>
      <BusTimeEntry route={'61D'} location={'Oakland'} time={13} busNumber={8888}/>
      <BusTimeEntry route={'61C'} location={'Downtown'} time={2} busNumber={6767}/>
    </div>
  </div>
</main>

<style>
  .stack {
    flex: 1 1 0;
    gap: 0.75rem;
  }
</style>
