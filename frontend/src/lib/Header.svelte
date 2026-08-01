<script lang="ts">
    import WeatherIcon from "~icons/wi/rain";
</script>

<div
    class="relative flex shrink-0 justify-between items-center bg-[#051B31] text-white font-semibold shadow-[0_3px_4px_0px_rgba(0,0,0,0.29)] overflow-hidden"
>
    <img
        class="pointer-events-none absolute inset-y-0 left-0 h-full w-auto"
        src="/header-left.png"
        alt=""
        aria-hidden="true"
    />
    <img
        class="pointer-events-none absolute inset-y-0 right-0 h-full w-auto"
        src="/header-right.png"
        alt=""
        aria-hidden="true"
    />
    <img
        class="relative z-10 h-18.5 w-18.5 my-4 ml-10"
        src="/cmu-wordmark-square-w-on-r.png"
        alt="CMU Logo"
    />
    <div
        class="absolute left-1/2 z-10 -translate-x-1/2 text-5xl whitespace-nowrap"
    >
        Live PRT Bus Arrival
    </div>
    <div class="relative z-10 flex flex-row items-center my-4.5 mr-14">
        <div class="flex flex-row items-center gap-3 pr-4.5">
            <WeatherIcon class="size-14" />
            <div class="gap-1">
                <div class="text-2xl">Heavy rain</div>
                <div class="text-sm">Until 10 PM</div>
            </div>
        </div>
        <div class="border-x-2 px-3 py-1.5">
            <div class="flex flex-col items-center">
                <div class="text-2xl">70 &deg;F</div>
                <div class="flex flex-row gap-2 text-sm">
                    <div>H 77&deg;</div>
                    <div>L 65&deg;</div>
                </div>
            </div>
        </div>
        <div class="gap-1 pl-4.5 text-xl">
            <div>06/15/26</div>
            <div>5:30 PM</div>
        </div>
    </div>
</div>
