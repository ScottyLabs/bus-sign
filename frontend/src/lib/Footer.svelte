<script lang="ts">
    import RefreshCW from "~icons/lucide/refresh-cw";

    export let lastUpdated: string = "";
</script>

<div
    class="flex shrink-0 justify-between items-center py-3 px-10 shadow-[0_-0.0625rem_0.25rem_0_rgba(0,0,0,0.13)]"
>
    <div class="flex flex-row gap-2 items-center">
        <RefreshCW class="size-4" />
        <div>Last updated: {lastUpdated}</div>
    </div>
    <div class="flex flex-row gap-2 items-center">
        <img
            class="h-[1.375rem] w-[1.5625rem]"
            src="/ScottyLabs.svg"
            alt="ScottyLabs Logo"
        />
        <div>
            Made by ScottyLabs in collaboration with the Undergraduate Student
            Senate
        </div>
    </div>
</div>
