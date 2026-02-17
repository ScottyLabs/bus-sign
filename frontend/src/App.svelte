<script lang="ts">
  import BusTimeEntry from './lib/BusTimeEntry.svelte';
  import QRcode from './assets/QRcode.png';

  let entriesUC = [
    { route: '61C', location: 'Downtown', time: 2, busNumber: 6767 },
    { route: '28X', location: 'Airporto', time: 11, busNumber: 1234 },
    { route: '61D', location: 'Oakland', time: 13, busNumber: 8888 },
    { route: '61A', location: 'CMU', time: 25, busNumber: 5032 },
  ];

  let entriesTep = [
    { route: '28X', location: 'Airporto', time: 11, busNumber: 1234 },
    { route: '61D', location: 'Oakland', time: 13, busNumber: 8888 },
    { route: '61C', location: 'Downtown', time: 67, busNumber: 6767 },
  ];
</script>

<main>
  <div class="container" style="justify-content: start; align-items: flex-start">
    <div class="stack left">
      <div style="font-size: 40px">
        UC Side (Stop 7117)
      </div>
      {#each entriesUC as entry (entry.busNumber)}
        <BusTimeEntry {...entry} />
      {:else}
        <BusTimeEntry route={'No Buses Running'} location={''} time={''} busNumber={''}/>
      {/each}
    </div>
    <div class="stack left">
      <div style="font-size: 40px">
        Tepper Side (Stop 4407)
      </div>
      {#each entriesTep as entry (entry.busNumber)}
        <BusTimeEntry {...entry} />
      {:else}
        <BusTimeEntry route={'No Buses Running'} location={''} time={''} busNumber={''}/>
      {/each}
    </div>
  </div>
  <div class="footer">
    <img src={QRcode} alt="QR code" />
  </div>
</main>

<style>
  .stack {
    flex: 1 1 0;
    gap: 0.75rem;
  }
  .footer {
    background-color: black;
    height: 20%;
  }

  img {
    width: auto;
    height: 100%;
    display: block;
    margin-left: auto;
  }
</style>
