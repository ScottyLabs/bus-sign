<script>
  export let route;
  export let location;
  export let time;
  export let busNumber;
</script>

<div class="rounded-box container">
  <div class="stack left">
    <div class="route">
      {route}
    </div>
    <div class="location">
      To {location.toUpperCase()}
    </div>
  </div>
  <div class="stack right">
    <div class="time">
      {time} MIN
    </div>
    <div class="bus-number">
      {busNumber}
    </div>
  </div>
</div>

<style>
  .rounded-box {
    border-radius: 16px;
    padding: 20px;
    background-color: #f0f0f0;
    border: 1px solid #ccc;
    min-width: 100%;
    min-height: 80px;
    box-sizing: border-box;
  }

  .route {
    font-size: 40px;
  }

  .time {
    font-size: 32px;
  }

  .location {

  }

  .bus-number {

  }
</style>
