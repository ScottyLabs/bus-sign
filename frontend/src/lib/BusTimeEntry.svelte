<script>
  export let route;
  export let location;
  export let time;
  export let busNumber;
</script>

<div class="rounded-box container">
  <div class="left-stack left">
    <div class="route">
      {route}
    </div>
    <div class="location">
      To {location.toUpperCase()}
    </div>
  </div>
  <div class="right-stack right">
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
    min-width: 100vw;
    min-height: 80px;
    box-sizing: border-box;
  }

  .container {
    display: flex;
    justify-content: space-between;
    align-items: center;

    width: 100%;
    box-sizing: border-box;
    padding: 1rem;
    border: 1px solid #ccc;
  }

  .left, .right {
    padding: 10px;
  }

  .left-stack {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }

  .right-stack {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
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
