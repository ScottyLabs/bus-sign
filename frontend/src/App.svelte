<script lang="ts">
  import BusTimeEntry from './lib/BusTimeEntry.svelte';
</script>

<main>
  <BusTimeEntry route={'61C'} location={'Downtown'} time={2} busNumber={6767}/>
  <BusTimeEntry route={'28X'} location={'Airporto'} time={11} busNumber={1234}/>
  <BusTimeEntry route={'61D'} location={'Oakland'} time={13} busNumber={8888}/>
  <BusTimeEntry route={'61A'} location={'CMU'} time={25} busNumber={5032}/>
</main>
