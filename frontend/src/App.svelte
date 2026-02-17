<script lang="ts">
  import BusTimeEntry from './lib/BusTimeEntry.svelte';
  import QRcode from './assets/QRcode.png';
</script>

<main>
  <div class="container" style="justify-content: start; align-items: flex-start">
    <div class="stack left">
      <div style="font-size: 40px">
        UC Side (Stop 7117)
      </div>
      <BusTimeEntry route={'61C'} location={'Downtown'} time={2} busNumber={6767}/>
      <BusTimeEntry route={'28X'} location={'Airporto'} time={11} busNumber={1234}/>
      <BusTimeEntry route={'61D'} location={'Oakland'} time={13} busNumber={8888}/>
      <BusTimeEntry route={'61A'} location={'CMU'} time={25} busNumber={5032}/>
    </div>
    <div class="stack left">
      <div style="font-size: 40px">
        Tepper Side (Stop 4407)
      </div>
      <BusTimeEntry route={'28X'} location={'Airporto'} time={11} busNumber={1234}/>
      <BusTimeEntry route={'61D'} location={'Oakland'} time={13} busNumber={8888}/>
      <BusTimeEntry route={'61C'} location={'Downtown'} time={2} busNumber={6767}/>
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
