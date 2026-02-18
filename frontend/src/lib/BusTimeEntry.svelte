<script lang="ts">
  export let route: string;
  export let destination: string;
  export let arrivals: {
    bus_id: string;
    capacity: string;
    seconds: number;
  }[];

  const formatTime = (seconds: number): string => {
    if (seconds < 30) return 'Approaching';
    const minutes = Math.ceil(seconds / 60);
    return `${minutes}`;
  };

  $: nextArrival = arrivals[0];
  $: timeDisplay = nextArrival ? formatTime(nextArrival.seconds) : 'N/A';
  $: upcomingTimes = arrivals.slice(1, 3).map(a => formatTime(a.seconds)).join(', ');
</script>

<div class="rounded-box container">
  <div class="stack left">
    <div class="route">
      {route}
    </div>
    <div>
      To {destination.toUpperCase()}
    </div>
  </div>
  <div class="stack right">
    <div class="time">
      {#if timeDisplay === 'Approaching'}
        {timeDisplay}
      {:else}
        {timeDisplay} MIN
      {/if}
    </div>
    <div>
      {#if upcomingTimes.length > 0}
        Next bus in {upcomingTimes} min
      {/if}
    </div>
  </div>
</div>

<style>
  .rounded-box {
    border-radius: 16px;
    padding: 8px;
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
</style>
