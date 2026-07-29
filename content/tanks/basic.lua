return {
    name = "Basic",
    health = 100,
    speed = 5,
    barrels = {
        { x = 0, y = 0, angle = 0, width = 10, length = 20 },
    },
    onShoot = function (player)
        player.barrels[1]:spawnBullet()
    end
}
